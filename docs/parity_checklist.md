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
- ✅ Finney runtime metadata captured (2024-XX-XX):
  - metadata_hash: `0x31a1392ead4c198c974610bc078f69346261648d306def22607e95fc521baf50`
  - spec_version: `377`
  - transaction_version: `1`
  - pallet indices: SubtensorModule=7, Commitments=18, Drand=26, System=0, Balances=5
- ✅ Subtensor extrinsics (call index + arg type IDs from metadata):
  - `set_weights` (pallet 7, call 0): `netuid:40`, `dests:146`, `weights:146`, `version_key:6`
  - `commit_weights` (pallet 7, call 96): `netuid:40`, `commit_hash:13`
  - `reveal_weights` (pallet 7, call 97): `netuid:40`, `uids:146`, `values:146`, `salt:146`, `version_key:6`
  - `commit_timelocked_weights` (pallet 7, call 113): `netuid:40`, `commit:152`, `reveal_round:6`, `commit_reveal_version:40`
  - `commit_timelocked_mechanism_weights` (pallet 7, call 118): `netuid:40`, `mecid:2`, `commit:152`, `reveal_round:6`, `commit_reveal_version:40`
  - `commit_crv3_mechanism_weights` (pallet 7, call 117): `netuid:40`, `mecid:2`, `commit:152`, `reveal_round:6`
  - `burned_register` (pallet 7, call 7): `netuid:40`, `hotkey:0`
  - `add_stake` (pallet 7, call 2): `hotkey:0`, `netuid:40`, `amount_staked:6`
  - `remove_stake` (pallet 7, call 3): `hotkey:0`, `netuid:40`, `amount_unstaked:6`
  - `transfer_keep_alive` (Balances pallet 5, call 3): `dest:141`, `value:12`
- ✅ `set_weights`/`commit_weights`/`reveal_weights` signatures verified against metadata. `uids`, `weights`, and `salt` use type id 146 (Vec<u16>), and `version_key` uses type id 6.
- ✅ `commit_timelocked_weights` and `commit_timelocked_mechanism_weights` include `commit_reveal_version` as final argument (type id 40).
- ✅ Transfer and staking extrinsics operate on RAO amounts on-chain (metadata type IDs 6/12 map to u128 balance types).

## Storage Indices & Keys
## Storage Indices & Keys
- ✅ Commit-reveal uses `NetUidStorageIndex` (u16) computed as `mech_id * 4096 + netuid`. Rust uses this in commit hash generation and timelocked storage queries.
- ✅ Storage reads for commitments use `SubtensorModule.CRV3WeightCommitsV2` and `TimelockedWeightCommits` with storage index keys.
- ✅ Storage key matrix (Finney metadata type IDs):
  - SubtensorModule.CommitRevealWeightsEnabled: key `40`, value `9`
  - SubtensorModule.CommitRevealWeightsVersion: key `None`, value `40`
  - SubtensorModule.RevealPeriodEpochs: key `40`, value `6`
  - SubtensorModule.CRV3WeightCommitsV2: key `450`, value `451`
  - SubtensorModule.TimelockedWeightCommits: key `450`, value `451`
  - SubtensorModule.CRV3WeightCommits: key `450`, value `453`
  - Commitments.RevealedCommitments: key `424`, value `501`
  - Commitments.CommitmentOf: key `424`, value `500`
  - Drand.LastStoredRound: key `None`, value `6`
  - Drand.BeaconConfig: key `None`, value `362`
  - System.Account: key `0`, value `3`
  - Balances.Account: key `0`, value `5`
- ✅ Drand storage expectations validated: `LastStoredRound` (type id 6) supplies the chain-relative drand round for CRv4 reveal calculations.
## Storage Indices & Keys
- ✅ Commit-reveal uses `NetUidStorageIndex` (u16) computed as `mech_id * 4096 + netuid`. Rust uses this in commit hash generation and timelocked storage queries.
- ✅ Storage reads for commitments use `SubtensorModule.CRV3WeightCommitsV2` and `TimelockedWeightCommits` with storage index keys.

## CRv4 Timelock (Commit-Reveal v4)
## CRv4 Timelock (Commit-Reveal v4)
- ✅ CRv4 flow uses chain `Drand.LastStoredRound`, tempo, reveal period, and commit-reveal version for reveal rounds.
- ✅ CRv4 persistence only tracks pending commits (auto-reveal on chain) and clears stale entries on epoch advances.
- ✅ Auto-reveal behavior documented: CRv4 commits are timelocked and automatically revealed on-chain; validators do not submit manual reveal extrinsics for CRv4.
- ✅ Timelock computation parameters documented: tempo is read from `SubtensorModule.Tempo`, reveal period from `SubtensorModule.RevealPeriodEpochs`, and drand state from `Drand.LastStoredRound`. Reveal round is calculated relative to the latest drand round.

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
- ✅ Docs reference Finney default entrypoint and runtime metadata hash/spec version above.
- ✅ `CommitRevealWeightsEnabled` default noted (true when storage missing).
- ✅ Tempo, rate limits, and CRv4 parameters are storage-backed (see storage matrix).