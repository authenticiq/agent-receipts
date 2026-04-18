# Conformance Test-Vector Matrix (Draft 1)

This matrix is frozen before verifier implementation begins.

## Conventions

- All fixture IDs are stable once published.
- All timestamps are UTC and fixed, never generated at test time.
- All receipt IDs use deterministic ULIDs.
- All content digests use lowercase hex.
- All ML-DSA-87 signatures are generated from fixed test keys.
- Where a file is intentionally malformed JSON, `schema-check` must fail before signature verification is attempted.

## Key fixtures

| Key ID | Purpose |
| --- | --- |
| `ml-dsa-87-primary` | Main valid signer for v1 fixtures |
| `ml-dsa-87-secondary` | Wrong-key negative tests |
| `ed25519-legacy` | Transitional compatibility tests |

## Valid vectors

| ID | Planned path | Scenario | Primary command | Expected result |
| --- | --- | --- | --- | --- |
| `V001` | `valid/minimal-single-ml-dsa-87.json` | Minimal single receipt with ML-DSA-87 signature | `receipts verify` | Pass |
| `V002` | `valid/reordered-json-fields.json` | Same semantic payload, non-canonical key order in file | `receipts verify` | Pass |
| `V003` | `valid/with-parent-receipt.json` | Receipt with `parent_receipt_id` set | `receipts verify` | Pass |
| `V004` | `valid/legacy-ed25519.json` | Transitional Ed25519 receipt | `receipts verify` | Pass |
| `V005` | `batches/two-receipts-valid-batch.json` | Two valid receipts with valid Merkle proofs | `receipts verify-batch` | Pass |
| `V006` | `chains/three-step-lineage.json` | Three-receipt causal chain with valid ancestry | `receipts verify-chain` | Pass |

## Invalid schema vectors

| ID | Planned path | Scenario | Primary command | Expected result |
| --- | --- | --- | --- | --- |
| `I001` | `invalid/missing-signature-field.json` | Signature object missing `value` | `receipts schema-check` | Fail |
| `I002` | `invalid/malformed-ulid.json` | Invalid `receipt_id` format | `receipts schema-check` | Fail |
| `I003` | `invalid/unsupported-signature-alg.json` | Unsupported algorithm name | `receipts schema-check` | Fail |
| `I004` | `invalid/malformed-base64-signature.json` | Signature `value` is not valid base64 | `receipts schema-check` | Fail |

## Invalid signature vectors

| ID | Planned path | Scenario | Primary command | Expected result |
| --- | --- | --- | --- | --- |
| `I005` | `invalid/tampered-input-hash.json` | `inputs_hash` changed after signing | `receipts verify` | Fail |
| `I006` | `invalid/tampered-output-hash.json` | `outputs_hash` changed after signing | `receipts verify` | Fail |
| `I007` | `invalid/wrong-public-key.json` | Valid receipt checked with wrong key fixture | `receipts verify` | Fail |
| `I008` | `invalid/signature-value-mutated.json` | Signature bytes changed but schema remains valid | `receipts verify` | Fail |

## Invalid lineage vectors

| ID | Planned path | Scenario | Primary command | Expected result |
| --- | --- | --- | --- | --- |
| `I009` | `chains/missing-parent-reference.json` | `parent_receipt_id` points to absent receipt | `receipts verify-chain` | Fail |
| `I010` | `chains/cycle-in-lineage.json` | Parent chain loops back on itself | `receipts verify-chain` | Fail |

## Invalid batch vectors

| ID | Planned path | Scenario | Primary command | Expected result |
| --- | --- | --- | --- | --- |
| `I011` | `batches/merkle-root-mismatch.json` | Root does not match reconstructed tree | `receipts verify-batch` | Fail |
| `I012` | `batches/proof-sibling-order-wrong.json` | Sibling ordering produces wrong root | `receipts verify-batch` | Fail |
| `I013` | `batches/leaf-hash-mismatch.json` | `leaf_hash` does not match nested receipt | `receipts verify-batch` | Fail |
| `I014` | `batches/receipt-count-mismatch.json` | `receipt_count` does not equal entry count | `receipts verify-batch` | Fail |
| `I015` | `batches/duplicate-receipt-id.json` | Same `receipt_id` appears twice in one batch | `receipts verify-batch` | Fail |

## Acceptance criteria for the first verifier

- The verifier MUST pass every `V*` fixture.
- The verifier MUST reject every `I*` fixture with a non-zero exit code.
- `schema-check` failures and cryptographic failures SHOULD produce different exit codes.
- The batch verifier MUST recompute leaf hashes from the nested receipts rather than trusting the provided `leaf_hash` field.
- The chain verifier MUST detect cycles and missing parents.