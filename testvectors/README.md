# Test Vectors

This directory will hold the conformance fixtures for `agent-receipts`.

The vector set is defined before verifier code starts so implementation work has a fixed contract to target.

## Planned layout

- `valid/` — receipts and batches that MUST verify successfully
- `invalid/` — receipts and batches that MUST fail verification
- `batches/` — standalone batch envelopes for inclusion-proof testing
- `chains/` — multi-receipt lineage fixtures for chain verification
- `keys/` — public key fixtures used by the vectors

See `MATRIX.md` for the frozen first-pass vector matrix.