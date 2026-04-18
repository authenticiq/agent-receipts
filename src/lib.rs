use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::{
    Signature as Ed25519Signature, SigningKey as Ed25519SigningKey,
    VerifyingKey as Ed25519VerifyingKey,
};
use ml_dsa::signature::Keypair as _;
use ml_dsa::{
    EncodedVerifyingKey, KeyGen as _, MlDsa87, Signature as MlDsaSignature,
    VerifyingKey as MlDsaVerifyingKey,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256, Sha512};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub const RECEIPT_SCHEMA_VERSION: &str = "agent-receipts/v1";
pub const BATCH_SCHEMA_VERSION: &str = "agent-receipts/batch/v1";
pub const KEY_FIXTURE_VERSION: &str = "agent-receipts/key-fixture/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    Agent,
    Human,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Mcp,
    Http,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignatureAlgorithm {
    MlDsa87,
    Ed25519,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub kind: ActorKind,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    pub transport: Transport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptPayload {
    pub event_type: String,
    pub actor: Actor,
    pub tool: Tool,
    pub inputs_hash: String,
    pub outputs_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_receipt_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureBlock {
    pub alg: SignatureAlgorithm,
    pub key_id: String,
    pub encoding: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub schema_version: String,
    pub receipt_id: String,
    pub issued_at: String,
    pub payload: ReceiptPayload,
    pub signature: SignatureBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    pub index: usize,
    pub siblings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchEntry {
    pub receipt: Receipt,
    pub leaf_hash: String,
    pub proof: MerkleProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Batch {
    pub schema_version: String,
    pub stratum_id: String,
    pub created_at: String,
    pub hash_algorithm: String,
    pub merkle_root: String,
    pub receipt_count: usize,
    pub entries: Vec<BatchEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainFixture {
    pub chain_id: String,
    pub receipts: Vec<Receipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKeyFixture {
    pub schema_version: String,
    pub key_id: String,
    pub alg: SignatureAlgorithm,
    pub encoding: String,
    pub value: String,
}

fn ulid_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[0-9A-HJKMNP-TV-Z]{26}$").expect("valid ULID regex"))
}

fn event_type_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^[a-z][a-z0-9_.-]{1,63}$").expect("valid event type regex"))
}

fn digest_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^(sha256:[a-f0-9]{64}|sha512:[a-f0-9]{128})$").expect("valid digest regex")
    })
}

pub fn validate_receipt(receipt: &Receipt) -> Result<()> {
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        bail!("unsupported schema_version: {}", receipt.schema_version);
    }

    if !ulid_regex().is_match(&receipt.receipt_id) {
        bail!("invalid receipt_id: {}", receipt.receipt_id);
    }

    OffsetDateTime::parse(&receipt.issued_at, &Rfc3339)
        .with_context(|| format!("invalid issued_at timestamp: {}", receipt.issued_at))?;

    if !event_type_regex().is_match(&receipt.payload.event_type) {
        bail!("invalid event_type: {}", receipt.payload.event_type);
    }

    validate_non_empty("actor.id", &receipt.payload.actor.id)?;
    validate_optional_string("actor.model", receipt.payload.actor.model.as_deref(), 256)?;
    validate_optional_string(
        "actor.session_id",
        receipt.payload.actor.session_id.as_deref(),
        256,
    )?;
    validate_non_empty("tool.name", &receipt.payload.tool.name)?;
    validate_optional_string("tool.version", receipt.payload.tool.version.as_deref(), 128)?;
    validate_optional_string("tool.server", receipt.payload.tool.server.as_deref(), 512)?;
    validate_digest("inputs_hash", &receipt.payload.inputs_hash)?;
    validate_digest("outputs_hash", &receipt.payload.outputs_hash)?;

    if let Some(parent_receipt_id) = &receipt.payload.parent_receipt_id {
        if !ulid_regex().is_match(parent_receipt_id) {
            bail!("invalid parent_receipt_id: {parent_receipt_id}");
        }
    }

    validate_non_empty("signature.key_id", &receipt.signature.key_id)?;
    if receipt.signature.encoding != "base64" {
        bail!(
            "unsupported signature encoding: {}",
            receipt.signature.encoding
        );
    }
    let signature_bytes = BASE64
        .decode(&receipt.signature.value)
        .with_context(|| "signature.value is not valid base64")?;
    if signature_bytes.is_empty() {
        bail!("signature.value decodes to empty bytes");
    }

    Ok(())
}

pub fn validate_batch(batch: &Batch) -> Result<()> {
    if batch.schema_version != BATCH_SCHEMA_VERSION {
        bail!("unsupported batch schema_version: {}", batch.schema_version);
    }

    OffsetDateTime::parse(&batch.created_at, &Rfc3339)
        .with_context(|| format!("invalid created_at timestamp: {}", batch.created_at))?;

    if batch.hash_algorithm != "sha512" {
        bail!("unsupported batch hash_algorithm: {}", batch.hash_algorithm);
    }

    validate_digest("merkle_root", &batch.merkle_root)?;

    if !batch.merkle_root.starts_with("sha512:") {
        bail!("batch merkle_root must use sha512");
    }

    if batch.receipt_count == 0 {
        bail!("batch receipt_count must be at least 1");
    }

    if batch.receipt_count != batch.entries.len() {
        bail!(
            "batch receipt_count {} does not match entry count {}",
            batch.receipt_count,
            batch.entries.len()
        );
    }

    for entry in &batch.entries {
        validate_receipt(&entry.receipt)?;
        validate_digest("leaf_hash", &entry.leaf_hash)?;
        if !entry.leaf_hash.starts_with("sha512:") {
            bail!("batch leaf_hash must use sha512");
        }
        for sibling in &entry.proof.siblings {
            validate_digest("proof sibling", sibling)?;
            if !sibling.starts_with("sha512:") {
                bail!("proof sibling must use sha512");
            }
        }
    }

    Ok(())
}

pub fn canonical_payload_bytes(payload: &ReceiptPayload) -> Result<Vec<u8>> {
    let value = serde_json::to_value(payload)?;
    canonical_json_bytes(&value)
}

pub fn canonical_receipt_bytes(receipt: &Receipt) -> Result<Vec<u8>> {
    let value = serde_json::to_value(receipt)?;
    canonical_json_bytes(&value)
}

pub fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>> {
    let sorted = sort_json_value(value);
    Ok(serde_json::to_vec(&sorted)?)
}

fn sort_json_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(sort_json_value).collect()),
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut sorted = Map::new();
            for key in keys {
                sorted.insert(key.clone(), sort_json_value(&map[&key]));
            }
            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

pub fn receipt_leaf_hash(receipt: &Receipt) -> Result<String> {
    Ok(sha512_digest_string(&canonical_receipt_bytes(receipt)?))
}

pub fn build_merkle_proofs(receipts: &[Receipt]) -> Result<(String, Vec<(String, MerkleProof)>)> {
    if receipts.is_empty() {
        bail!("cannot build Merkle proofs for empty receipt set");
    }

    let leaves = receipts
        .iter()
        .map(receipt_leaf_hash)
        .collect::<Result<Vec<_>>>()?;

    let root = merkle_root_for_leaves(&leaves)?;
    let proofs = (0..leaves.len())
        .map(|index| {
            Ok((
                leaves[index].clone(),
                MerkleProof {
                    index,
                    siblings: merkle_siblings(&leaves, index)?,
                },
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((root, proofs))
}

pub fn verify_receipt(
    receipt: &Receipt,
    public_keys: &HashMap<String, PublicKeyFixture>,
) -> Result<()> {
    validate_receipt(receipt)?;

    let key_fixture = public_keys
        .get(&receipt.signature.key_id)
        .with_context(|| format!("unknown key_id: {}", receipt.signature.key_id))?;

    if key_fixture.alg != receipt.signature.alg {
        bail!(
            "signature algorithm mismatch: receipt uses {:?}, key fixture uses {:?}",
            receipt.signature.alg,
            key_fixture.alg
        );
    }

    if key_fixture.schema_version != KEY_FIXTURE_VERSION {
        bail!(
            "unsupported key fixture schema_version: {}",
            key_fixture.schema_version
        );
    }

    if key_fixture.encoding != "base64" {
        bail!("unsupported key fixture encoding: {}", key_fixture.encoding);
    }

    let message = canonical_payload_bytes(&receipt.payload)?;
    let key_bytes = BASE64
        .decode(&key_fixture.value)
        .with_context(|| format!("invalid base64 in key fixture {}", key_fixture.key_id))?;
    let signature_bytes = BASE64
        .decode(&receipt.signature.value)
        .with_context(|| format!("invalid base64 in signature for {}", receipt.receipt_id))?;

    match receipt.signature.alg {
        SignatureAlgorithm::MlDsa87 => {
            verify_ml_dsa_signature(&message, &key_bytes, &signature_bytes)
        }
        SignatureAlgorithm::Ed25519 => {
            verify_ed25519_signature(&message, &key_bytes, &signature_bytes)
        }
    }
}

pub fn verify_batch(batch: &Batch, public_keys: &HashMap<String, PublicKeyFixture>) -> Result<()> {
    validate_batch(batch)?;

    let mut receipt_ids = HashSet::new();
    let mut indexed_leaves: Vec<Option<String>> = vec![None; batch.receipt_count];

    for entry in &batch.entries {
        if !receipt_ids.insert(entry.receipt.receipt_id.clone()) {
            bail!(
                "duplicate receipt_id in batch: {}",
                entry.receipt.receipt_id
            );
        }

        if entry.proof.index >= batch.receipt_count {
            bail!(
                "proof index {} out of bounds for batch count {}",
                entry.proof.index,
                batch.receipt_count
            );
        }

        verify_receipt(&entry.receipt, public_keys)?;

        let expected_leaf_hash = receipt_leaf_hash(&entry.receipt)?;
        if expected_leaf_hash != entry.leaf_hash {
            bail!(
                "leaf hash mismatch for receipt {}: expected {}, got {}",
                entry.receipt.receipt_id,
                expected_leaf_hash,
                entry.leaf_hash
            );
        }

        if !verify_merkle_proof(&entry.leaf_hash, &entry.proof, &batch.merkle_root) {
            bail!(
                "Merkle proof failed for receipt {}",
                entry.receipt.receipt_id
            );
        }

        if indexed_leaves[entry.proof.index].is_some() {
            bail!("duplicate proof index in batch: {}", entry.proof.index);
        }
        indexed_leaves[entry.proof.index] = Some(entry.leaf_hash.clone());
    }

    let leaves = indexed_leaves
        .into_iter()
        .map(|leaf| leaf.ok_or_else(|| anyhow!("missing leaf for one or more proof indexes")))
        .collect::<Result<Vec<_>>>()?;

    let recomputed_root = merkle_root_for_leaves(&leaves)?;
    if recomputed_root != batch.merkle_root {
        bail!(
            "batch merkle_root mismatch: expected {}, got {}",
            batch.merkle_root,
            recomputed_root
        );
    }

    Ok(())
}

pub fn verify_chain(
    chain: &ChainFixture,
    public_keys: &HashMap<String, PublicKeyFixture>,
) -> Result<()> {
    if chain.receipts.is_empty() {
        bail!("chain fixture contains no receipts");
    }

    let mut receipts_by_id = HashMap::new();
    for receipt in &chain.receipts {
        verify_receipt(receipt, public_keys)?;
        if receipts_by_id
            .insert(receipt.receipt_id.clone(), receipt)
            .is_some()
        {
            bail!("duplicate receipt_id in chain: {}", receipt.receipt_id);
        }
    }

    for receipt in &chain.receipts {
        if let Some(parent_id) = &receipt.payload.parent_receipt_id {
            if !receipts_by_id.contains_key(parent_id) {
                bail!(
                    "receipt {} references missing parent {}",
                    receipt.receipt_id,
                    parent_id
                );
            }
        }
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for receipt in &chain.receipts {
        detect_cycle(
            &receipt.receipt_id,
            &receipts_by_id,
            &mut visiting,
            &mut visited,
        )?;
    }

    Ok(())
}

fn detect_cycle<'a>(
    receipt_id: &str,
    receipts_by_id: &HashMap<String, &'a Receipt>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> Result<()> {
    if visited.contains(receipt_id) {
        return Ok(());
    }

    if !visiting.insert(receipt_id.to_string()) {
        bail!("cycle detected at receipt {}", receipt_id);
    }

    if let Some(parent_id) = receipts_by_id[receipt_id]
        .payload
        .parent_receipt_id
        .as_deref()
    {
        detect_cycle(parent_id, receipts_by_id, visiting, visited)?;
    }

    visiting.remove(receipt_id);
    visited.insert(receipt_id.to_string());
    Ok(())
}

fn verify_ml_dsa_signature(message: &[u8], key_bytes: &[u8], signature_bytes: &[u8]) -> Result<()> {
    use ml_dsa::signature::Verifier as _;

    let encoded_key = EncodedVerifyingKey::<MlDsa87>::try_from(key_bytes)
        .map_err(|_| anyhow!("invalid ML-DSA verifying key length"))?;
    let verifying_key = MlDsaVerifyingKey::<MlDsa87>::decode(&encoded_key);
    let signature = MlDsaSignature::<MlDsa87>::try_from(signature_bytes)
        .map_err(|_| anyhow!("invalid ML-DSA signature encoding"))?;

    verifying_key
        .verify(message, &signature)
        .map_err(|_| anyhow!("ML-DSA signature verification failed"))
}

fn verify_ed25519_signature(
    message: &[u8],
    key_bytes: &[u8],
    signature_bytes: &[u8],
) -> Result<()> {
    use ed25519_dalek::Verifier as _;

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow!("invalid Ed25519 public key length"))?;
    let verifying_key = Ed25519VerifyingKey::from_bytes(&key_array)
        .map_err(|_| anyhow!("invalid Ed25519 public key bytes"))?;
    let signature = Ed25519Signature::try_from(signature_bytes)
        .map_err(|_| anyhow!("invalid Ed25519 signature encoding"))?;

    verifying_key
        .verify(message, &signature)
        .map_err(|_| anyhow!("Ed25519 signature verification failed"))
}

pub fn load_public_keys_dir(path: &Path) -> Result<HashMap<String, PublicKeyFixture>> {
    let mut fixtures = HashMap::new();

    for entry in fs::read_dir(path)
        .with_context(|| format!("failed to read keys directory {}", path.display()))?
    {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let fixture: PublicKeyFixture = read_json_file(&entry_path)?;
        fixtures.insert(fixture.key_id.clone(), fixture);
    }

    if fixtures.is_empty() {
        bail!("no public key fixtures found in {}", path.display());
    }

    Ok(fixtures)
}

pub fn schema_check_file(path: &Path) -> Result<String> {
    let value: Value = read_json_file(path)?;
    let schema_version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("document is missing schema_version"))?;

    match schema_version {
        RECEIPT_SCHEMA_VERSION => {
            let receipt: Receipt = serde_json::from_value(value)
                .with_context(|| format!("{} is not a valid receipt document", path.display()))?;
            validate_receipt(&receipt)?;
            Ok(RECEIPT_SCHEMA_VERSION.to_string())
        }
        BATCH_SCHEMA_VERSION => {
            let batch: Batch = serde_json::from_value(value)
                .with_context(|| format!("{} is not a valid batch document", path.display()))?;
            validate_batch(&batch)?;
            Ok(BATCH_SCHEMA_VERSION.to_string())
        }
        other => bail!("unsupported schema_version: {other}"),
    }
}

pub fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse JSON in {}", path.display()))
}

pub fn write_pretty_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut content = serde_json::to_string_pretty(value)?;
    content.push('\n');
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

pub fn sha256_digest_string(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub fn sha512_digest_string(bytes: &[u8]) -> String {
    format!("sha512:{}", hex::encode(Sha512::digest(bytes)))
}

pub fn write_string(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

pub fn batch_fixture_from_receipts(
    stratum_id: &str,
    created_at: &str,
    receipts: &[Receipt],
) -> Result<Batch> {
    let (merkle_root, proofs) = build_merkle_proofs(receipts)?;
    let entries = receipts
        .iter()
        .cloned()
        .zip(proofs)
        .map(|(receipt, (leaf_hash, proof))| BatchEntry {
            receipt,
            leaf_hash,
            proof,
        })
        .collect::<Vec<_>>();

    Ok(Batch {
        schema_version: BATCH_SCHEMA_VERSION.to_string(),
        stratum_id: stratum_id.to_string(),
        created_at: created_at.to_string(),
        hash_algorithm: "sha512".to_string(),
        merkle_root,
        receipt_count: entries.len(),
        entries,
    })
}

pub fn default_keys_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testvectors/keys")
}

pub fn sign_receipt_payload_ml_dsa(
    seed: [u8; 32],
    key_id: &str,
    receipt_id: &str,
    issued_at: &str,
    payload: ReceiptPayload,
) -> Result<(Receipt, PublicKeyFixture)> {
    use ml_dsa::signature::Signer as _;

    let signing_key = MlDsa87::from_seed(&seed.into());
    let payload_bytes = canonical_payload_bytes(&payload)?;
    let signature = signing_key.sign(&payload_bytes);
    let public_key = signing_key.verifying_key().encode();

    Ok((
        Receipt {
            schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: receipt_id.to_string(),
            issued_at: issued_at.to_string(),
            payload,
            signature: SignatureBlock {
                alg: SignatureAlgorithm::MlDsa87,
                key_id: key_id.to_string(),
                encoding: "base64".to_string(),
                value: BASE64.encode(signature.encode().as_slice()),
            },
        },
        PublicKeyFixture {
            schema_version: KEY_FIXTURE_VERSION.to_string(),
            key_id: key_id.to_string(),
            alg: SignatureAlgorithm::MlDsa87,
            encoding: "base64".to_string(),
            value: BASE64.encode(public_key.as_slice()),
        },
    ))
}

pub fn sign_receipt_payload_ed25519(
    secret_key: [u8; 32],
    key_id: &str,
    receipt_id: &str,
    issued_at: &str,
    payload: ReceiptPayload,
) -> Result<(Receipt, PublicKeyFixture)> {
    use ed25519_dalek::Signer as _;

    let signing_key = Ed25519SigningKey::from_bytes(&secret_key);
    let payload_bytes = canonical_payload_bytes(&payload)?;
    let signature = signing_key.sign(&payload_bytes);

    Ok((
        Receipt {
            schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: receipt_id.to_string(),
            issued_at: issued_at.to_string(),
            payload,
            signature: SignatureBlock {
                alg: SignatureAlgorithm::Ed25519,
                key_id: key_id.to_string(),
                encoding: "base64".to_string(),
                value: BASE64.encode(signature.to_bytes()),
            },
        },
        PublicKeyFixture {
            schema_version: KEY_FIXTURE_VERSION.to_string(),
            key_id: key_id.to_string(),
            alg: SignatureAlgorithm::Ed25519,
            encoding: "base64".to_string(),
            value: BASE64.encode(signing_key.verifying_key().to_bytes()),
        },
    ))
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{field_name} must not be empty");
    }
    if value.len() > 256 {
        bail!("{field_name} exceeds maximum length");
    }
    Ok(())
}

fn validate_optional_string(
    field_name: &str,
    value: Option<&str>,
    max_length: usize,
) -> Result<()> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            bail!("{field_name} must not be empty when present");
        }
        if value.len() > max_length {
            bail!("{field_name} exceeds maximum length of {max_length}");
        }
    }
    Ok(())
}

fn validate_digest(field_name: &str, value: &str) -> Result<()> {
    if !digest_regex().is_match(value) {
        bail!("invalid {field_name}: {value}");
    }
    Ok(())
}

fn merkle_root_for_leaves(leaves: &[String]) -> Result<String> {
    if leaves.is_empty() {
        bail!("cannot compute Merkle root for zero leaves");
    }

    let mut current = leaves.to_vec();
    while current.len() > 1 {
        let mut next = Vec::new();
        let mut index = 0;
        while index < current.len() {
            let left = &current[index];
            let right = current.get(index + 1).unwrap_or(left);
            next.push(hash_pair(left, right));
            index += 2;
        }
        current = next;
    }

    Ok(current[0].clone())
}

fn merkle_siblings(leaves: &[String], target_index: usize) -> Result<Vec<String>> {
    if target_index >= leaves.len() {
        bail!("Merkle proof index {target_index} out of bounds");
    }

    let mut siblings = Vec::new();
    let mut current = leaves.to_vec();
    let mut index = target_index;

    while current.len() > 1 {
        let pair_index = if index % 2 == 0 { index + 1 } else { index - 1 };
        let sibling = current.get(pair_index).unwrap_or(&current[index]).clone();
        siblings.push(sibling);

        let mut next = Vec::new();
        let mut layer_index = 0;
        while layer_index < current.len() {
            let left = &current[layer_index];
            let right = current.get(layer_index + 1).unwrap_or(left);
            next.push(hash_pair(left, right));
            layer_index += 2;
        }

        current = next;
        index /= 2;
    }

    Ok(siblings)
}

fn verify_merkle_proof(leaf_hash: &str, proof: &MerkleProof, expected_root: &str) -> bool {
    let mut current = leaf_hash.to_string();
    let mut index = proof.index;

    for sibling in &proof.siblings {
        current = if index % 2 == 0 {
            hash_pair(&current, sibling)
        } else {
            hash_pair(sibling, &current)
        };
        index /= 2;
    }

    current == expected_root
}

fn hash_pair(left: &str, right: &str) -> String {
    sha512_digest_string(format!("{left}{right}").as_bytes())
}
