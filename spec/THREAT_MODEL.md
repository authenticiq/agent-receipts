# Receipt v1 Threat Model

This document captures the security assumptions and limits of `agent-receipts` v1.

## Security goals

- Detect tampering with signed agent-action claims.
- Let independent verifiers validate receipts offline.
- Let independent verifiers validate Merkle inclusion proofs offline.
- Avoid hidden dependence on a hosted control plane.

## Assets protected

- Integrity of the signed `payload`
- Integrity of the detached signature object
- Integrity of Merkle inclusion proofs in `batch-v1`
- Portability of verification across different operators

## Trust boundaries

### Receipt issuer

The issuer creates the payload and applies the signature. If the issuer lies before signing, the receipt can preserve the lie perfectly. The schema is not a truth oracle.

### Key owner

The verifier trusts a public key because of out-of-band trust policy. v1 does not define how a verifier decides which keys are trustworthy.

### Batcher / ledger operator

The batcher organizes receipts into Merkle trees. The batcher can omit receipts, delay publication, or present alternate ledgers unless additional transparency constraints are layered on top.

### Verifier

The verifier is responsible for canonicalization, signature validation, and proof reconstruction. A buggy verifier can accept malformed receipts.

## Attacker model

The model assumes attackers may:

- Modify receipt files at rest or in transit
- Reorder JSON object keys to exploit non-canonical parsers
- Swap signatures, key identifiers, or proof paths
- Replace a receipt with one signed by a different key
- Tamper with Merkle roots or proof siblings
- Replay valid receipts in misleading contexts
- Attempt to create hosted-service dependency assumptions in tooling or docs

The model does not assume an attacker can break ML-DSA-87 or Ed25519 cryptographically.

## What v1 is intended to detect

### Payload tampering

If any signed field inside `payload` changes, signature verification should fail.

### Canonicalization abuse

If a verifier uses the canonicalization rules in `RATIONALE.md`, reordering keys should not change the signed bytes.

### Wrong-key substitution

If a receipt is verified with the wrong public key, verification should fail.

### Batch or proof tampering

If a receipt's Merkle path, leaf hash, or root is modified, batch verification should fail.

## What v1 does not prove

### Semantic correctness

The receipt does not prove that the input or output content was correct, safe, policy-compliant, or truthful. It proves integrity of what was signed.

### Real-world identity

`actor.id` identifies an actor inside the issuer's trust domain. It does not, by itself, prove a legal, human, or organizational identity.

### Completeness of history

An operator can omit receipts entirely. A valid receipt does not prove that it is the only receipt, the first receipt, or a complete log of actions.

### Trusted time

`issued_at` is the issuer's asserted timestamp. It is not a trusted external timestamp unless paired with another system.

### Authorization or consent

The receipt does not prove the action was authorized. It only proves that a signer asserted it happened.

## Operational guidance

- Prefer ML-DSA-87 for new issuers.
- Treat Ed25519 as transitional compatibility only.
- Publish or distribute trusted public keys through a separate, auditable mechanism.
- Use `batch-v1` or a public ledger when completeness or inclusion matters.
- Keep raw inputs and outputs outside the receipt if they contain sensitive data; only their digests belong in the schema.

## Future hardening paths

- Witnessed batch checkpoints
- Public-key transparency or revocation mechanisms
- External timestamping
- Multi-party signing or witness cosigning
- Formal conformance suites across multiple implementations