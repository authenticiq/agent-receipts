# Integration Guide

This guide is for teams that want to emit `agent-receipts` from their own agent runtime, tool server, or audit pipeline.

## Current integration surface

- The shipped integration surface today is the JSON schema contract plus the Rust verifier CLI and library.
- TypeScript and Python bindings are planned, but they are not published yet.
- If you are not integrating from Rust yet, the easiest path is to emit schema-conformant JSON and verify it with the Rust CLI in CI or in your ingest pipeline.

## What an emitter must produce

A valid receipt needs:

- `schema_version`: `agent-receipts/v1`
- `receipt_id`: ULID
- `issued_at`: RFC 3339 timestamp
- `payload.event_type`: open vocabulary such as `tool_call`
- `payload.actor`: who acted
- `payload.tool`: what execution surface handled the action
- `payload.inputs_hash` and `payload.outputs_hash`: content digests, not raw payload bodies
- `signature`: detached signature metadata including algorithm, key identifier, encoding, and value

Merkle inclusion data does not belong inside the signed receipt. If you batch receipts, emit a separate `batch-v1` envelope rather than mutating the receipt after signing.

## Recommended emission flow

1. Capture stable input and output bytes before any downstream formatting changes.
2. Hash those bytes with `sha256` or `sha512`.
3. Build the `payload` object.
4. Canonicalize and sign the `payload` only.
5. Write the receipt JSON plus a public-key fixture JSON file.
6. Verify the emitted receipt with the CLI before treating the integration as trustworthy.

## Rust reference example

The current helper APIs make Rust the cleanest integration path today:

```rust
use anyhow::Result;
use agent_receipts::{
    Actor, ActorKind, ReceiptPayload, Tool, Transport, sha256_digest_string,
    sign_receipt_payload_ml_dsa,
};
use std::fs;

fn main() -> Result<()> {
    let input_bytes = br#"{"path":"README.md"}"#;
    let output_bytes = br#"{"bytes":1024}"#;

    let payload = ReceiptPayload {
        event_type: "tool_call".to_string(),
        actor: Actor {
            kind: ActorKind::Agent,
            id: "agent:demo-runner".to_string(),
            model: Some("claude-sonnet-4".to_string()),
            session_id: Some("session:demo-002".to_string()),
        },
        tool: Tool {
            name: "filesystem.read".to_string(),
            version: Some("1.0.0".to_string()),
            server: Some("mcp://demo.local/fs".to_string()),
            transport: Transport::Mcp,
        },
        inputs_hash: sha256_digest_string(input_bytes),
        outputs_hash: sha256_digest_string(output_bytes),
        parent_receipt_id: None,
    };

    // Demo-only fixed seed. Replace with your real key custody in production.
    let (receipt, public_key) = sign_receipt_payload_ml_dsa(
        [7u8; 32],
        "demo-ml-dsa-87",
        "01JABCD0000000000000000000",
        "2026-04-18T18:05:00Z",
        payload,
    )?;

    fs::create_dir_all("demo-keys")?;
    fs::write("demo-receipt.json", serde_json::to_vec_pretty(&receipt)?)?;
    fs::write(
        "demo-keys/demo-ml-dsa-87.public.json",
        serde_json::to_vec_pretty(&public_key)?,
    )?;

    Ok(())
}
```

Then verify what you emitted:

```bash
receipts verify demo-receipt.json --keys-dir demo-keys
```

## Non-Rust integrations today

If your runtime is not Rust yet:

- emit JSON that conforms to [spec/receipt-v1.schema.json](../spec/receipt-v1.schema.json)
- emit public key fixture JSON shaped like the files in [testvectors/keys](../testvectors/keys)
- run `receipts schema-check` and `receipts verify` in CI or in a validation step before you ship receipts downstream

That keeps the public contract stable even before language-specific bindings land.

## Batches and chains

- Use `parent_receipt_id` when you want to preserve causal lineage between receipts.
- Use `batch-v1` when you need Merkle inclusion proofs for a set of receipts.
- Do not add batch metadata to an already signed receipt. Inclusion is a separate envelope.

## Operational notes

- Prefer ML-DSA-87 for new integrations.
- Treat Ed25519 as transitional compatibility only.
- Publish or distribute trusted public keys separately from the receipt itself.
- The demo signing helpers are useful for reference implementations and fixtures, but production key custody remains your responsibility.

## Validation checklist

- Your receipts pass `receipts schema-check`.
- Your receipts pass `receipts verify` against the intended public key set.
- Your integration preserves stable byte capture for input and output hashing.
- Your batching layer emits `batch-v1` instead of mutating signed receipts.