# Quickstart

This guide gets you from clone to verified receipt in under 10 minutes using the committed fixtures in this repo.

## Prerequisites

- `git`
- Rust toolchain installed locally

## 1. Clone the repo and inspect the CLI

```bash
git clone https://github.com/authenticiq/agent-receipts.git
cd agent-receipts
cargo run --bin receipts -- --help
```

You should see the available commands:

```text
schema-check
verify
verify-batch
verify-chain
inspect
```

## 2. Schema-check and verify a single receipt

```bash
cargo run --bin receipts -- schema-check testvectors/valid/minimal-single-ml-dsa-87.json
cargo run --bin receipts -- verify testvectors/valid/minimal-single-ml-dsa-87.json
```

Expected output:

```text
schema-check ok: agent-receipts/v1
verify ok: 01JABCD0000000000000000000
```

Those commands succeed because the repo already contains the matching public keys in `testvectors/keys` and the CLI uses that directory by default when you run from this checkout.

## 3. Inspect the receipt body

```bash
cargo run --bin receipts -- inspect testvectors/valid/minimal-single-ml-dsa-87.json
```

This pretty-prints the receipt so you can see the signed `payload`, detached `signature`, and top-level envelope metadata.

## 4. Verify the batch fixture

```bash
cargo run --bin receipts -- verify-batch testvectors/batches/two-receipts-valid-batch.json
```

Expected output:

```text
verify-batch ok: demo-stratum-2026-04-18
```

This recomputes leaf hashes and validates the Merkle root and proof paths rather than trusting the batch file blindly.

## 5. Verify the lineage fixture

```bash
cargo run --bin receipts -- verify-chain testvectors/chains/three-step-lineage.json
```

Expected output:

```text
verify-chain ok: demo-chain-v1
```

This checks parent-child relationships and rejects missing-parent or cycle fixtures.

## 6. See a failure mode

```bash
cargo run --bin receipts -- verify testvectors/invalid/tampered-output-hash.json
```

That command should exit non-zero because the signed payload was altered after signing.

## 7. Verify your own files

When you move beyond the repo fixtures, pass the public key directory explicitly:

```bash
receipts verify path/to/receipt.json --keys-dir path/to/public-keys
receipts verify-batch path/to/batch.json --keys-dir path/to/public-keys
receipts verify-chain path/to/chain.json --keys-dir path/to/public-keys
```

The `--keys-dir` directory should contain one or more public key fixture JSON files shaped like the examples in `testvectors/keys/`.

## Next steps

- Read [docs/INTEGRATION.md](INTEGRATION.md) to emit your own receipts.
- Read [spec/RATIONALE.md](../spec/RATIONALE.md) for the design rules that signers and verifiers are expected to preserve.
- Read [testvectors/MATRIX.md](../testvectors/MATRIX.md) for the current conformance suite contract.