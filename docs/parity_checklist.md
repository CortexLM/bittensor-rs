# Bittensor Rust SDK Parity Checklist

This checklist captures parity gaps between the Rust SDK and the upstream Python SDK/runtime expectations. Items are grouped by domain and annotated with current status and required changes.

## Status Legend
- ✅ Implemented and validated
- ⚠️ Implemented but parity risk or missing validation
- ❌ Missing or incomplete (requires work)

## RAO/TAO Semantics & Units
- ⚠️ RAO/TAO conversions: Rust has `Rao`/`Tao` newtypes and helpers, but docs still show raw `u128` in multiple APIs. Ensure every public-facing API documents RAO vs TAO explicitly and that conversions match Python SDK rounding (truncate toward zero). Update docs to reflect RAO-only on-chain.
- ⚠️ Emission units: `subnets::subnet_info` sums `Emission` and converts to TAO (`f64`). Python SDK treats emissions as RAO or TAO depending on call. Validate that Rust uses RAO internally and only formats to TAO for display.
- ⚠️ Stake/balance presentation: docs and examples frequently show TAO formatted values. Ensure the SDK stores u128 RAO and uses formatting helpers for display only.

## Extrinsic Signatures & Parameters
- ⚠️ `set_weights`/`commit_weights`/`reveal_weights` signatures should match Subtensor exactly (uids/weights as `Vec<u16>`, `version_key`, and `salt: Vec<u16>`). Confirm extrinsics use correct SCALE types and storage index (NetUidStorageIndex) as per upstream.
- ⚠️ `commit_timelocked_weights` and `commit_timelocked_mechanism_weights` include `commit_reveal_version` argument. Verify Python SDK default and runtime versioning; keep in sync.
- ⚠️ `transfer` and staking extrinsics operate in RAO amounts. Confirm docs avoid mixing TAO and RAO.

## Storage Indices & Keys
- ✅ Commit-reveal uses `NetUidStorageIndex` (u16) computed as `mech_id * 4096 + netuid`. Rust uses this in commit hash generation and timelocked storage queries.
- ✅ Storage reads for commitments use `SubtensorModule.CRV3WeightCommitsV2` and `TimelockedWeightCommits` with storage index keys.

## CRv4 Timelock (Commit-Reveal v4)
- ✅ CRv4 flow uses chain `Drand.LastStoredRound`, tempo, reveal period, and commit-reveal version for reveal rounds.
- ✅ CRv4 persistence only tracks pending commits (auto-reveal on chain) and clears stale entries on epoch advances.
- ⚠️ Ensure auto-reveal behavior is documented: CRv4 requires no manual reveal, and incorrect reveal rounds must be handled.

## Missing/Incomplete APIs (vs Python SDK)
- ❌ Wallet parity: Python SDK includes wallet utils for coldkey/hotkey management, mnemonic generation, and keystore behaviors. Validate Rust wallet feature coverage and document gaps.
- ❌ Delegate and governance operations: verify exposure of senate/voting, proposal management, and delegate APIs aligned with Python SDK.
- ❌ Metagraph and runtime API queries: Python SDK exposes runtime API endpoints (metagraph, neuron info) with caching; document any missing or partial support.
- ❌ Dendrite/Axon integration parity: Python SDK includes network stack details (priority, timeout, endpoints). Verify Rust equivalents and document missing features.

## Validation & Runtime Expectations
- ⚠️ Ensure docs reference finney default entrypoint and chain runtime expectations (Subtensor). Confirm all storage indices and extrinsics align with latest runtime metadata.
- ⚠️ Ensure `commit_reveal_enabled` defaults align with runtime behavior (current docs should note default true when storage missing).
- ⚠️ Confirm any on-chain constants (tempo, weights rate limits) are read from storage not hardcoded.