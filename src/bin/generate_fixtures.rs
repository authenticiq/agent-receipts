use std::path::PathBuf;

use agent_receipts::{
    Actor, ActorKind, Batch, ChainFixture, Receipt, ReceiptPayload, Tool, Transport,
    batch_fixture_from_receipts, sign_receipt_payload_ed25519, sign_receipt_payload_ml_dsa,
    write_pretty_json, write_string,
};
use anyhow::Result;
use base64::Engine;
use serde_json::json;

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let testvectors = root.join("testvectors");

    let (minimal_receipt, primary_key) = sign_receipt_payload_ml_dsa(
        [1_u8; 32],
        "ml-dsa-87-primary",
        "01JABCD0000000000000000000",
        "2026-04-18T18:00:00Z",
        ReceiptPayload {
            event_type: "tool_call".to_string(),
            actor: Actor {
                kind: ActorKind::Agent,
                id: "agent:demo-runner".to_string(),
                model: Some("claude-sonnet-4".to_string()),
                session_id: Some("session:demo-001".to_string()),
            },
            tool: Tool {
                name: "filesystem.read".to_string(),
                version: Some("1.0.0".to_string()),
                server: Some("mcp://demo.local/fs".to_string()),
                transport: Transport::Mcp,
            },
            inputs_hash: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
            outputs_hash: "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                .to_string(),
            parent_receipt_id: None,
        },
    )?;

    let (_, secondary_key) = sign_receipt_payload_ml_dsa(
        [2_u8; 32],
        "ml-dsa-87-secondary",
        "01JABCD000000000000000000A",
        "2026-04-18T18:00:01Z",
        ReceiptPayload {
            event_type: "tool_call".to_string(),
            actor: Actor {
                kind: ActorKind::Agent,
                id: "agent:secondary".to_string(),
                model: Some("claude-sonnet-4".to_string()),
                session_id: Some("session:demo-002".to_string()),
            },
            tool: Tool {
                name: "filesystem.read".to_string(),
                version: Some("1.0.0".to_string()),
                server: Some("mcp://demo.local/fs".to_string()),
                transport: Transport::Mcp,
            },
            inputs_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            outputs_hash: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_string(),
            parent_receipt_id: None,
        },
    )?;

    let (legacy_receipt, legacy_key) = sign_receipt_payload_ed25519(
        [3_u8; 32],
        "ed25519-legacy",
        "01JABCD0000000000000000003",
        "2026-04-18T18:00:03Z",
        ReceiptPayload {
            event_type: "tool_call".to_string(),
            actor: Actor {
                kind: ActorKind::Human,
                id: "user:reviewer".to_string(),
                model: None,
                session_id: Some("session:demo-003".to_string()),
            },
            tool: Tool {
                name: "editor.annotate".to_string(),
                version: Some("0.9.0".to_string()),
                server: Some("local://editor".to_string()),
                transport: Transport::Local,
            },
            inputs_hash: "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                .to_string(),
            outputs_hash: "sha256:4444444444444444444444444444444444444444444444444444444444444444"
                .to_string(),
            parent_receipt_id: None,
        },
    )?;

    let (parented_receipt, _) = sign_receipt_payload_ml_dsa(
        [1_u8; 32],
        "ml-dsa-87-primary",
        "01JABCD0000000000000000001",
        "2026-04-18T18:00:01Z",
        ReceiptPayload {
            event_type: "handoff".to_string(),
            actor: Actor {
                kind: ActorKind::Agent,
                id: "agent:demo-runner".to_string(),
                model: Some("claude-sonnet-4".to_string()),
                session_id: Some("session:demo-001".to_string()),
            },
            tool: Tool {
                name: "planner.route".to_string(),
                version: Some("1.2.0".to_string()),
                server: Some("mcp://demo.local/planner".to_string()),
                transport: Transport::Mcp,
            },
            inputs_hash: "sha256:5555555555555555555555555555555555555555555555555555555555555555"
                .to_string(),
            outputs_hash: "sha256:6666666666666666666666666666666666666666666666666666666666666666"
                .to_string(),
            parent_receipt_id: Some(minimal_receipt.receipt_id.clone()),
        },
    )?;

    let (third_chain_receipt, _) = sign_receipt_payload_ed25519(
        [3_u8; 32],
        "ed25519-legacy",
        "01JABCD0000000000000000002",
        "2026-04-18T18:00:02Z",
        ReceiptPayload {
            event_type: "review".to_string(),
            actor: Actor {
                kind: ActorKind::Human,
                id: "user:reviewer".to_string(),
                model: None,
                session_id: Some("session:demo-003".to_string()),
            },
            tool: Tool {
                name: "review.queue".to_string(),
                version: Some("2.0.0".to_string()),
                server: Some("http://localhost/review".to_string()),
                transport: Transport::Http,
            },
            inputs_hash: "sha256:7777777777777777777777777777777777777777777777777777777777777777"
                .to_string(),
            outputs_hash: "sha256:8888888888888888888888888888888888888888888888888888888888888888"
                .to_string(),
            parent_receipt_id: Some(parented_receipt.receipt_id.clone()),
        },
    )?;

    write_pretty_json(
        &testvectors.join("keys/ml-dsa-87-primary.public.json"),
        &primary_key,
    )?;
    write_pretty_json(
        &testvectors.join("keys/ml-dsa-87-secondary.public.json"),
        &secondary_key,
    )?;
    write_pretty_json(
        &testvectors.join("keys/ed25519-legacy.public.json"),
        &legacy_key,
    )?;
    write_pretty_json(
        &testvectors.join("valid/minimal-single-ml-dsa-87.json"),
        &minimal_receipt,
    )?;
    write_reordered_receipt_exact(
        &testvectors.join("valid/reordered-json-fields.json"),
        &minimal_receipt,
    )?;
    write_pretty_json(
        &testvectors.join("valid/with-parent-receipt.json"),
        &parented_receipt,
    )?;
    write_pretty_json(
        &testvectors.join("valid/legacy-ed25519.json"),
        &legacy_receipt,
    )?;

    let batch = batch_fixture_from_receipts(
        "demo-stratum-2026-04-18",
        "2026-04-18T19:00:00Z",
        &[minimal_receipt.clone(), parented_receipt.clone()],
    )?;
    write_pretty_json(
        &testvectors.join("batches/two-receipts-valid-batch.json"),
        &batch,
    )?;

    let chain = ChainFixture {
        chain_id: "demo-chain-v1".to_string(),
        receipts: vec![
            minimal_receipt.clone(),
            parented_receipt.clone(),
            third_chain_receipt.clone(),
        ],
    };
    write_pretty_json(&testvectors.join("chains/three-step-lineage.json"), &chain)?;

    write_invalid_schema_fixtures(&testvectors, &minimal_receipt)?;
    write_invalid_signature_fixtures(&testvectors, &minimal_receipt)?;
    write_invalid_chain_fixtures(
        &testvectors,
        &minimal_receipt,
        &parented_receipt,
        &third_chain_receipt,
    )?;
    write_invalid_batch_fixtures(&testvectors, &batch)?;

    Ok(())
}

fn write_invalid_schema_fixtures(
    testvectors: &std::path::Path,
    minimal_receipt: &Receipt,
) -> Result<()> {
    let mut missing_signature_value = serde_json::to_value(minimal_receipt)?;
    missing_signature_value
        .get_mut("signature")
        .and_then(|value| value.as_object_mut())
        .expect("signature object")
        .remove("value");
    write_pretty_json(
        &testvectors.join("invalid/missing-signature-field.json"),
        &missing_signature_value,
    )?;

    let mut malformed_ulid = serde_json::to_value(minimal_receipt)?;
    malformed_ulid["receipt_id"] = json!("not-a-valid-ulid");
    write_pretty_json(
        &testvectors.join("invalid/malformed-ulid.json"),
        &malformed_ulid,
    )?;

    let mut unsupported_alg = serde_json::to_value(minimal_receipt)?;
    unsupported_alg["signature"]["alg"] = json!("rsa-4096");
    write_pretty_json(
        &testvectors.join("invalid/unsupported-signature-alg.json"),
        &unsupported_alg,
    )?;

    let mut malformed_base64 = serde_json::to_value(minimal_receipt)?;
    malformed_base64["signature"]["value"] = json!("%%%not-base64%%%");
    write_pretty_json(
        &testvectors.join("invalid/malformed-base64-signature.json"),
        &malformed_base64,
    )?;
    Ok(())
}

fn write_invalid_signature_fixtures(
    testvectors: &std::path::Path,
    minimal_receipt: &Receipt,
) -> Result<()> {
    let mut tampered_input = minimal_receipt.clone();
    tampered_input.payload.inputs_hash =
        "sha256:9999999999999999999999999999999999999999999999999999999999999999".to_string();
    write_pretty_json(
        &testvectors.join("invalid/tampered-input-hash.json"),
        &tampered_input,
    )?;

    let mut tampered_output = minimal_receipt.clone();
    tampered_output.payload.outputs_hash =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
    write_pretty_json(
        &testvectors.join("invalid/tampered-output-hash.json"),
        &tampered_output,
    )?;

    let mut wrong_key = minimal_receipt.clone();
    wrong_key.signature.key_id = "ml-dsa-87-secondary".to_string();
    write_pretty_json(
        &testvectors.join("invalid/wrong-public-key.json"),
        &wrong_key,
    )?;

    let mut mutated_signature = minimal_receipt.clone();
    let mut signature_bytes =
        base64::engine::general_purpose::STANDARD.decode(&mutated_signature.signature.value)?;
    signature_bytes[0] ^= 0x01;
    mutated_signature.signature.value =
        base64::engine::general_purpose::STANDARD.encode(signature_bytes);
    write_pretty_json(
        &testvectors.join("invalid/signature-value-mutated.json"),
        &mutated_signature,
    )?;
    Ok(())
}

fn write_invalid_chain_fixtures(
    testvectors: &std::path::Path,
    minimal_receipt: &Receipt,
    parented_receipt: &Receipt,
    third_chain_receipt: &Receipt,
) -> Result<()> {
    let mut missing_parent = third_chain_receipt.clone();
    missing_parent.payload.parent_receipt_id = Some("01JABCD000000000000000000Z".to_string());
    let missing_parent_fixture = ChainFixture {
        chain_id: "missing-parent".to_string(),
        receipts: vec![
            minimal_receipt.clone(),
            parented_receipt.clone(),
            missing_parent,
        ],
    };
    write_pretty_json(
        &testvectors.join("chains/missing-parent-reference.json"),
        &missing_parent_fixture,
    )?;

    let (cycle_receipt_one, _) = sign_receipt_payload_ml_dsa(
        [1_u8; 32],
        "ml-dsa-87-primary",
        "01JABCD0000000000000000004",
        "2026-04-18T18:01:00Z",
        ReceiptPayload {
            event_type: "handoff".to_string(),
            actor: Actor {
                kind: ActorKind::Agent,
                id: "agent:cycle-a".to_string(),
                model: Some("claude-sonnet-4".to_string()),
                session_id: Some("session:cycle".to_string()),
            },
            tool: Tool {
                name: "planner.route".to_string(),
                version: Some("1.2.0".to_string()),
                server: Some("mcp://demo.local/planner".to_string()),
                transport: Transport::Mcp,
            },
            inputs_hash: "sha256:1212121212121212121212121212121212121212121212121212121212121212"
                .to_string(),
            outputs_hash: "sha256:3434343434343434343434343434343434343434343434343434343434343434"
                .to_string(),
            parent_receipt_id: Some("01JABCD0000000000000000005".to_string()),
        },
    )?;

    let (cycle_receipt_two, _) = sign_receipt_payload_ml_dsa(
        [1_u8; 32],
        "ml-dsa-87-primary",
        "01JABCD0000000000000000005",
        "2026-04-18T18:01:01Z",
        ReceiptPayload {
            event_type: "handoff".to_string(),
            actor: Actor {
                kind: ActorKind::Agent,
                id: "agent:cycle-b".to_string(),
                model: Some("claude-sonnet-4".to_string()),
                session_id: Some("session:cycle".to_string()),
            },
            tool: Tool {
                name: "planner.route".to_string(),
                version: Some("1.2.0".to_string()),
                server: Some("mcp://demo.local/planner".to_string()),
                transport: Transport::Mcp,
            },
            inputs_hash: "sha256:5656565656565656565656565656565656565656565656565656565656565656"
                .to_string(),
            outputs_hash: "sha256:7878787878787878787878787878787878787878787878787878787878787878"
                .to_string(),
            parent_receipt_id: Some("01JABCD0000000000000000004".to_string()),
        },
    )?;

    let cycle_fixture = ChainFixture {
        chain_id: "cycle".to_string(),
        receipts: vec![cycle_receipt_one, cycle_receipt_two],
    };
    write_pretty_json(
        &testvectors.join("chains/cycle-in-lineage.json"),
        &cycle_fixture,
    )?;
    Ok(())
}

fn write_invalid_batch_fixtures(testvectors: &std::path::Path, valid_batch: &Batch) -> Result<()> {
    let mut root_mismatch = valid_batch.clone();
    root_mismatch.merkle_root = "sha512:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
    write_pretty_json(
        &testvectors.join("batches/merkle-root-mismatch.json"),
        &root_mismatch,
    )?;

    let mut wrong_sibling_order = valid_batch.clone();
    if let Some(entry) = wrong_sibling_order.entries.first_mut() {
        if let Some(sibling) = entry.proof.siblings.first_mut() {
            *sibling = "sha512:cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd".to_string();
        }
    }
    write_pretty_json(
        &testvectors.join("batches/proof-sibling-order-wrong.json"),
        &wrong_sibling_order,
    )?;

    let mut leaf_hash_mismatch = valid_batch.clone();
    if let Some(entry) = leaf_hash_mismatch.entries.first_mut() {
        entry.leaf_hash = "sha512:abababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababab".to_string();
    }
    write_pretty_json(
        &testvectors.join("batches/leaf-hash-mismatch.json"),
        &leaf_hash_mismatch,
    )?;

    let mut receipt_count_mismatch = valid_batch.clone();
    receipt_count_mismatch.receipt_count += 1;
    write_pretty_json(
        &testvectors.join("batches/receipt-count-mismatch.json"),
        &receipt_count_mismatch,
    )?;

    let mut duplicate_receipt_id = valid_batch.clone();
    if duplicate_receipt_id.entries.len() == 2 {
        duplicate_receipt_id.entries[1].receipt.receipt_id =
            duplicate_receipt_id.entries[0].receipt.receipt_id.clone();
    }
    write_pretty_json(
        &testvectors.join("batches/duplicate-receipt-id.json"),
        &duplicate_receipt_id,
    )?;
    Ok(())
}

fn write_reordered_receipt_exact(path: &std::path::Path, receipt: &Receipt) -> Result<()> {
    let signature_alg = serde_json::to_string(&receipt.signature.alg)?;
    let actor_kind = serde_json::to_string(&receipt.payload.actor.kind)?;
    let tool_transport = serde_json::to_string(&receipt.payload.tool.transport)?;

    let content = format!(
        concat!(
            "{{\n",
            "  \"signature\": {{\n",
            "    \"value\": \"{}\",\n",
            "    \"encoding\": \"{}\",\n",
            "    \"key_id\": \"{}\",\n",
            "    \"alg\": {}\n",
            "  }},\n",
            "  \"payload\": {{\n",
            "    \"outputs_hash\": \"{}\",\n",
            "    \"tool\": {{\n",
            "      \"transport\": {},\n",
            "      \"server\": \"{}\",\n",
            "      \"version\": \"{}\",\n",
            "      \"name\": \"{}\"\n",
            "    }},\n",
            "    \"inputs_hash\": \"{}\",\n",
            "    \"actor\": {{\n",
            "      \"session_id\": \"{}\",\n",
            "      \"model\": \"{}\",\n",
            "      \"id\": \"{}\",\n",
            "      \"kind\": {}\n",
            "    }},\n",
            "    \"event_type\": \"{}\"\n",
            "  }},\n",
            "  \"issued_at\": \"{}\",\n",
            "  \"receipt_id\": \"{}\",\n",
            "  \"schema_version\": \"{}\"\n",
            "}}\n"
        ),
        receipt.signature.value,
        receipt.signature.encoding,
        receipt.signature.key_id,
        signature_alg,
        receipt.payload.outputs_hash,
        tool_transport,
        receipt.payload.tool.server.as_deref().unwrap_or_default(),
        receipt.payload.tool.version.as_deref().unwrap_or_default(),
        receipt.payload.tool.name,
        receipt.payload.inputs_hash,
        receipt
            .payload
            .actor
            .session_id
            .as_deref()
            .unwrap_or_default(),
        receipt.payload.actor.model.as_deref().unwrap_or_default(),
        receipt.payload.actor.id,
        actor_kind,
        receipt.payload.event_type,
        receipt.issued_at,
        receipt.receipt_id,
        receipt.schema_version,
    );

    write_string(path, &content)
}
