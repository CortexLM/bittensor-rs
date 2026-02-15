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
- ⚠️ Missing metadata-backed table for Subtensor extrinsics (call index, arg order, SCALE types). Acceptance: add a table sourced from runtime metadata (`runtime.rs`/subxt) and record the finney runtime spec/metadata hash used for validation.
- ⚠️ `set_weights`/`commit_weights`/`reveal_weights` signatures should match Subtensor exactly (`uids`/`weights` as `Vec<u16>`, `version_key`, `salt: Vec<u16>`). Acceptance: verify against metadata (call index + arg types) and Python SDK signature, and document the verified signature.
- ⚠️ `commit_timelocked_weights` and `commit_timelocked_mechanism_weights` include `commit_reveal_version` argument. Acceptance: confirm arg ordering/types + default commit reveal version from runtime storage/constants and document expected value.
- ⚠️ `transfer` and staking extrinsics operate in RAO amounts. Acceptance: document RAO-only amounts in all extrinsic docs and verify encoding types are `u128` in metadata.

## Storage Indices & Keys
- ✅ Commit-reveal uses `NetUidStorageIndex` (u16) computed as `mech_id * 4096 + netuid`. Rust uses this in commit hash generation and timelocked storage queries.
- ✅ Storage reads for commitments use `SubtensorModule.CRV3WeightCommitsV2` and `TimelockedWeightCommits` with storage index keys.
- ⚠️ Storage key coverage is undocumented for required pallets (SubtensorModule, Drand, System). Acceptance: add a storage key matrix listing each storage item read, key types, and value types as reported by runtime metadata.
- ⚠️ Drand storage expectations are not validated (last stored round, public key/config). Acceptance: confirm storage item names and key types from metadata and document required fields for CRv4 computation.
## Storage Indices & Keys
- ✅ Commit-reveal uses `NetUidStorageIndex` (u16) computed as `mech_id * 4096 + netuid`. Rust uses this in commit hash generation and timelocked storage queries.
- ✅ Storage reads for commitments use `SubtensorModule.CRV3WeightCommitsV2` and `TimelockedWeightCommits` with storage index keys.

## CRv4 Timelock (Commit-Reveal v4)
- ✅ CRv4 flow uses chain `Drand.LastStoredRound`, tempo, reveal period, and commit-reveal version for reveal rounds.
- ✅ CRv4 persistence only tracks pending commits (auto-reveal on chain) and clears stale entries on epoch advances.
- ⚠️ Ensure auto-reveal behavior is documented: CRv4 requires no manual reveal, and incorrect reveal rounds must be handled.
- ⚠️ Timelock computation parameters are not verified against metadata constants. Acceptance: document the exact formula (epoch, reveal period, drand round) and cite the runtime storage/constant names used for each value.

## Missing/Incomplete APIs (vs Python SDK)
- ❌ Wallet parity: Python SDK includes wallet utils for coldkey/hotkey management, mnemonic generation, and keystore behaviors. Validate Rust wallet feature coverage and document gaps.
- ⚠️ Delegate and governance operations: senate voting and proposal calls now exposed. Validate runtime call metadata and update CLI coverage.
- ⚠️ Metagraph and runtime API queries: metagraph snapshots and runtime helpers implemented. Add cache strategies and parity validation against Python SDK.
- ✅ Dendrite/Axon integration parity: Dendrite/Axon includes header parity, timeout handling, priority/blacklist middleware, and streaming support.
- ❌ Wallet compatibility specifics are not tracked (derivation path, keystore JSON schema, encryption defaults, SS58 format). Acceptance: document current Rust behavior vs Python SDK for mnemonic length, derivation paths, and keystore compatibility.
## Missing/Incomplete APIs (vs Python SDK)
- ❌ Wallet parity: Python SDK includes wallet utils for coldkey/hotkey management, mnemonic generation, and keystore behaviors. Validate Rust wallet feature coverage and document gaps.
- ⚠️ Delegate and governance operations: senate voting and proposal calls now exposed. Validate runtime call metadata and update CLI coverage.
- ⚠️ Metagraph and runtime API queries: metagraph snapshots and runtime helpers implemented. Add cache strategies and parity validation against Python SDK.
- ✅ Dendrite/Axon integration parity: Dendrite/Axon includes header parity, timeout handling, priority/blacklist middleware, and streaming support.

## Validation & Runtime Expectations
- ⚠️ Ensure docs reference finney default entrypoint and chain runtime expectations (Subtensor). Confirm all storage indices and extrinsics align with latest runtime metadata.
- ⚠️ Ensure `commit_reveal_enabled` defaults align with runtime behavior (current docs should note default true when storage missing).
- ⚠️ Confirm any on-chain constants (tempo, weights rate limits) are read from storage not hardcoded.