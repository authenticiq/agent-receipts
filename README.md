# agent-receipts

Post-quantum receipt schema and verifier for AI agent actions.

`agent-receipts` defines a neutral JSON contract for signed action receipts and Merkle inclusion batches, plus a Rust verifier CLI and library that validate them offline. It is maintained by AuthenticIQ. StrataCodes is a downstream commercial implementation, not the parent brand of the OSS and not a required service dependency.

Current surface area:
- `receipt-v1` and `batch-v1` schemas
- Rust verifier CLI and library
- conformance fixtures in `testvectors/`
- ML-DSA-87 as the preferred algorithm, with Ed25519 kept as transitional compatibility
- no packaged release binaries or TS/Python bindings yet

## Why you'd use it

- Agent platforms and MCP server authors: emit portable, signed receipts for tool calls without depending on a hosted control plane.
- Compliance and audit teams: verify a claim offline from a receipt file, a batch file, and a trusted public key directory.
- Researchers and journalists: preserve tamper-evident evidence that an agent action was asserted by a specific signer.

## Install

The fastest path today is from a local clone. That keeps the demo fixtures and public keys in place and makes the first verification run copy-pasteable.

```bash
git clone https://github.com/authenticiq/agent-receipts.git
cd agent-receipts
cargo run --bin receipts -- --help
```

If you want the CLI on your local `PATH`, install it from the checked-out repo:

```bash
cargo install --locked --path .
receipts --help
```

The built-in demo path assumes a checked-out repo because the default key directory points at `testvectors/keys`. If you verify your own receipts or run the binary outside this checkout, pass `--keys-dir` explicitly.

## Verify your first receipt

From the repo root:

```bash
cargo run --bin receipts -- schema-check testvectors/valid/minimal-single-ml-dsa-87.json
cargo run --bin receipts -- verify testvectors/valid/minimal-single-ml-dsa-87.json
cargo run --bin receipts -- inspect testvectors/valid/minimal-single-ml-dsa-87.json
```

You should see:

```text
schema-check ok: agent-receipts/v1
verify ok: 01JABCD0000000000000000000
```

For the next two verifier surfaces:

```bash
cargo run --bin receipts -- verify-batch testvectors/batches/two-receipts-valid-batch.json
cargo run --bin receipts -- verify-chain testvectors/chains/three-step-lineage.json
```

See [docs/QUICKSTART.md](docs/QUICKSTART.md) for the 10-minute path, expected outputs, and a failure demo.

## How it relates to adjacent tools

| System | Primary object | What it is good at | Relationship to `agent-receipts` |
| --- | --- | --- | --- |
| `agent-receipts` | Runtime agent actions | Signed action receipts and offline verification | This repo is the contract and reference verifier |
| Sigstore | Build and release artifacts | Artifact signing, identity, and release provenance | Complementary; not a runtime action format |
| in-toto | Supply-chain steps and attestations | Build pipeline claims and policy evaluation | Complementary; broader supply-chain focus |
| C2PA | Media assets | Content credentials and media provenance | Complementary; not aimed at tool-call receipts |
| OpenTelemetry | Traces, logs, and metrics | Operational observability | Complementary; not tamper-evident by itself |

## Spec and conformance

- Receipt schema: [spec/receipt-v1.schema.json](spec/receipt-v1.schema.json)
- Batch schema: [spec/batch-v1.schema.json](spec/batch-v1.schema.json)
- Design rationale: [spec/RATIONALE.md](spec/RATIONALE.md)
- Threat model: [spec/THREAT_MODEL.md](spec/THREAT_MODEL.md)
- Conformance matrix: [testvectors/MATRIX.md](testvectors/MATRIX.md)
- Quickstart: [docs/QUICKSTART.md](docs/QUICKSTART.md)
- Integration guide: [docs/INTEGRATION.md](docs/INTEGRATION.md)

## Contributing and security

This project uses DCO sign-off rather than a CLA. See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution rules and review expectations.

Do not open public issues for vulnerabilities. Report security issues privately to `hello@authenticiq.ai` as described in [SECURITY.md](SECURITY.md).