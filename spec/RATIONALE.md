# Receipt v1 Rationale

This document explains the design choices behind `receipt-v1.schema.json` and `batch-v1.schema.json`.

## Goals

- Define a neutral, portable receipt format for AI agent actions.
- Make verification possible offline with no product dependency.
- Keep the signed unit small, deterministic, and implementation-friendly.
- Separate action receipts from product-specific workflow concepts such as internal "codes".
- Provide a batch format that can travel independently of any hosted ledger.

## Non-goals for v1

- No requirement for a StrataCodes account, API, or hosted control plane.
- No attempt to standardize key distribution, revocation, or trust policy.
- No claim that receipts prove semantic correctness, policy compliance, or user intent.
- No product taxonomy for vertical workflows.

## Why the receipt uses `payload` plus `signature`

The cryptographically signed claims live inside `payload`. The `signature` object is detached metadata that proves integrity over the canonical JSON serialization of `payload` only.

This envelope structure resolves a common ambiguity in attestation formats: data that is added after signing should not mutate the signed object. In particular, Merkle inclusion data belongs in a batch envelope, not inside the signed receipt payload.

Normative rule for v1:

- Signers MUST sign the canonical JSON serialization of `payload` only.
- Verifiers MUST recompute the signature input from `payload` only.
- `receipt_id`, `issued_at`, and `schema_version` are envelope metadata and are not sufficient on their own to verify integrity.

## Why inclusion data is kept out of `receipt-v1`

Receipts are often created before the final batch or ledger stratum exists. If inclusion fields were part of the signed receipt, a batcher would either have to mutate a signed object or require a second signing step.

v1 avoids that complexity:

- `receipt-v1` proves the action claim.
- `batch-v1` proves inclusion in a Merkle batch.

That split keeps receipts stable across transports and lets different operators batch them differently without re-issuing signatures.

## Field choices

### `receipt_id`

ULID is used because it is sortable, compact, and easy to generate across languages. It also avoids coupling the identifier format to a database or hosted service.

### `issued_at`

`issued_at` records the issuer's clock. It is useful for ordering and diagnostics, but it is not a trusted external timestamp authority on its own. Operators that need stronger time assurances should layer external timestamping or transparency infrastructure on top.

### `event_type`

`event_type` uses an open vocabulary rather than a closed enum. That keeps the OSS format neutral and avoids freezing product-specific categories into the public contract.

### `actor`

The actor model is intentionally small: `kind`, `id`, optional `model`, optional `session_id`. It captures who acted without forcing a particular identity system.

### `tool`

The tool object records the execution surface: name, optional version, optional server identifier, and transport (`mcp`, `http`, or `local`). This makes the receipt useful across MCP-native and non-MCP systems.

### `inputs_hash` and `outputs_hash`

The receipt stores digests, not raw inputs or outputs. This keeps the format compact, privacy-preserving, and portable. Actual payloads may remain local, private, or referenced elsewhere.

v1 digest strings use `<algorithm>:<lowercase-hex>` and allow `sha256` or `sha512` for content hashes. Merkle batches standardize on `sha512` for tree construction.

### Signature algorithms

`ml-dsa-87` is the preferred algorithm for v1 because the project is intentionally post-quantum forward-looking. `ed25519` remains permitted as a transitional compatibility algorithm during early adoption.

## Canonicalization rules

The verifier MUST canonicalize `payload` using these rules before signing or verification:

- Encode as UTF-8 JSON.
- Sort object keys lexicographically at every object level.
- Preserve array order exactly as supplied.
- Do not add whitespace beyond what is required by JSON string encoding.
- Do not normalize string values beyond standard JSON escaping.

If two files contain semantically identical payload objects but with different key order, they MUST verify to the same signature input.

## Batch design

`batch-v1` is a portable envelope that bundles:

- the signed receipt,
- the leaf hash used in the Merkle tree,
- the receipt's proof path.

This makes offline verification straightforward. A verifier does not need a network service if it has the batch file and the relevant public key material.

## Relationship to adjacent standards

### Sigstore / in-toto

Those systems focus on software artifacts, builds, and attestations in the software supply chain. `agent-receipts` focuses on runtime agent actions.

### C2PA

C2PA focuses on media provenance and content credentials. `agent-receipts` focuses on action provenance and operational auditability.

### OpenTelemetry

OpenTelemetry is excellent for traces and observability, but it does not define a tamper-evident receipt format. `agent-receipts` is complementary, not competitive.

## Open issues for v1 review

- Whether `issued_at` should eventually move into `payload` in a future version.
- Whether a standardized verification-material extension is needed for self-contained bundles.
- Whether batch checkpoints should be standardized in a separate schema rather than inside `batch-v1`.