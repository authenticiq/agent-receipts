# agent-receipts Guidelines

This repo is the contract surface for the receipts OSS stack.

## What this repo is

- schema repo for `receipt-v1` and `batch-v1`
- verifier CLI and library
- conformance fixture source of truth
- release trust surface for downstream repos

## High-sensitivity surfaces

- schemas in `spec/`
- canonicalization and hashing behavior
- signature verification behavior
- generated fixtures in `testvectors/`
- CLI behavior that users depend on in automation

## Before making changes

1. Read `README.md`, `spec/RATIONALE.md`, `spec/THREAT_MODEL.md`, and `testvectors/MATRIX.md`.
2. Inspect current workflows and the working tree.
3. Treat schema, fixture, and verifier changes as contract changes, not ordinary refactors.

## Required validation

- `cargo fmt --all --check`
- `cargo run --bin generate-fixtures`
- confirm a clean working tree after fixture regeneration
- `cargo test`
- `../.tools/gitleaks dir . --config gitleaks.toml`

## Repo rules

- Do not hand-edit generated fixtures unless there is a documented reason and the generator is updated too.
- If behavior changes, update specs, fixtures, tests, and usage docs in the same pass.
- Keep OSS neutral. StrataCodes is a downstream commercial implementation, not the parent brand.
- ML-DSA is the preferred path; Ed25519 support is transitional and should be kept clearly marked.