use std::path::Path;

use agent_receipts::{
    Batch, ChainFixture, Receipt, default_keys_dir, load_public_keys_dir, read_json_file,
    schema_check_file, verify_batch, verify_chain, verify_receipt,
};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn schema_check_passes_for_primary_valid_receipt() {
    let path = repo_root().join("testvectors/valid/minimal-single-ml-dsa-87.json");
    let schema_version = schema_check_file(&path).expect("valid receipt schema-check should pass");
    assert_eq!(schema_version, "agent-receipts/v1");
}

#[test]
fn verify_passes_for_primary_valid_receipt() {
    let keys = load_public_keys_dir(&default_keys_dir()).expect("load key fixtures");
    let receipt: Receipt =
        read_json_file(&repo_root().join("testvectors/valid/minimal-single-ml-dsa-87.json"))
            .expect("load receipt fixture");
    verify_receipt(&receipt, &keys).expect("primary ML-DSA receipt should verify");
}

#[test]
fn verify_batch_passes_for_valid_batch() {
    let keys = load_public_keys_dir(&default_keys_dir()).expect("load key fixtures");
    let batch: Batch =
        read_json_file(&repo_root().join("testvectors/batches/two-receipts-valid-batch.json"))
            .expect("load batch fixture");
    verify_batch(&batch, &keys).expect("valid batch should verify");
}

#[test]
fn verify_chain_passes_for_valid_chain() {
    let keys = load_public_keys_dir(&default_keys_dir()).expect("load key fixtures");
    let chain: ChainFixture =
        read_json_file(&repo_root().join("testvectors/chains/three-step-lineage.json"))
            .expect("load chain fixture");
    verify_chain(&chain, &keys).expect("valid chain should verify");
}

#[test]
fn schema_check_rejects_invalid_schema_fixture() {
    let path = repo_root().join("testvectors/invalid/unsupported-signature-alg.json");
    assert!(schema_check_file(&path).is_err());
}

#[test]
fn verify_rejects_wrong_public_key_fixture() {
    let keys = load_public_keys_dir(&default_keys_dir()).expect("load key fixtures");
    let receipt: Receipt =
        read_json_file(&repo_root().join("testvectors/invalid/wrong-public-key.json"))
            .expect("load invalid receipt fixture");
    assert!(verify_receipt(&receipt, &keys).is_err());
}

#[test]
fn verify_batch_rejects_duplicate_receipt_ids() {
    let keys = load_public_keys_dir(&default_keys_dir()).expect("load key fixtures");
    let batch: Batch =
        read_json_file(&repo_root().join("testvectors/batches/duplicate-receipt-id.json"))
            .expect("load invalid batch fixture");
    assert!(verify_batch(&batch, &keys).is_err());
}

#[test]
fn verify_chain_rejects_cycle_fixture() {
    let keys = load_public_keys_dir(&default_keys_dir()).expect("load key fixtures");
    let chain: ChainFixture =
        read_json_file(&repo_root().join("testvectors/chains/cycle-in-lineage.json"))
            .expect("load invalid chain fixture");
    assert!(verify_chain(&chain, &keys).is_err());
}
