#!/usr/bin/env bash

set -euo pipefail

cargo build --locked --bin receipts

receipts_cmd=(target/debug/receipts)

run_expect_success() {
  local label=$1
  shift

  echo "::group::${label}"
  "${receipts_cmd[@]}" "$@"
  echo "::endgroup::"
}

run_expect_failure() {
  local label=$1
  shift

  echo "::group::${label}"
  if "${receipts_cmd[@]}" "$@"; then
    echo "expected failure for ${label}" >&2
    exit 1
  fi
  echo "::endgroup::"
}

valid_receipts=(
  testvectors/valid/minimal-single-ml-dsa-87.json
  testvectors/valid/reordered-json-fields.json
  testvectors/valid/with-parent-receipt.json
  testvectors/valid/legacy-ed25519.json
)

invalid_schema_receipts=(
  testvectors/invalid/missing-signature-field.json
  testvectors/invalid/malformed-ulid.json
  testvectors/invalid/unsupported-signature-alg.json
  testvectors/invalid/malformed-base64-signature.json
)

invalid_signature_receipts=(
  testvectors/invalid/tampered-input-hash.json
  testvectors/invalid/tampered-output-hash.json
  testvectors/invalid/wrong-public-key.json
  testvectors/invalid/signature-value-mutated.json
)

valid_batches=(
  testvectors/batches/two-receipts-valid-batch.json
)

invalid_batches=(
  testvectors/batches/merkle-root-mismatch.json
  testvectors/batches/proof-sibling-order-wrong.json
  testvectors/batches/leaf-hash-mismatch.json
  testvectors/batches/receipt-count-mismatch.json
  testvectors/batches/duplicate-receipt-id.json
)

valid_chains=(
  testvectors/chains/three-step-lineage.json
)

invalid_chains=(
  testvectors/chains/missing-parent-reference.json
  testvectors/chains/cycle-in-lineage.json
)

for path in "${valid_receipts[@]}"; do
  run_expect_success "schema-check ${path}" schema-check "$path"
done

for path in "${invalid_schema_receipts[@]}"; do
  run_expect_failure "schema-check ${path}" schema-check "$path"
done

for path in "${valid_receipts[@]}"; do
  run_expect_success "verify ${path}" verify "$path"
done

for path in "${invalid_signature_receipts[@]}"; do
  run_expect_failure "verify ${path}" verify "$path"
done

for path in "${valid_batches[@]}"; do
  run_expect_success "verify-batch ${path}" verify-batch "$path"
done

for path in "${invalid_batches[@]}"; do
  run_expect_failure "verify-batch ${path}" verify-batch "$path"
done

for path in "${valid_chains[@]}"; do
  run_expect_success "verify-chain ${path}" verify-chain "$path"
done

for path in "${invalid_chains[@]}"; do
  run_expect_failure "verify-chain ${path}" verify-chain "$path"
done