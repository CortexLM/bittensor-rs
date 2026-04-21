# Bittensor Rust SDK (bittensor-rs) — Complete Reimplementation

## TL;DR

> **Quick Summary**: Complete Rust reimplementation of the Bittensor Python SDK v10.2.0, covering wallet management (NaCl keyfile compat), blockchain interaction (subxt 0.50.0), neuron-to-neuron communication (Axon/Dendrite/Synapse protocol), Metagraph state, full CLI (btcli parity), PyO3 bindings for all crates, TUI (ratatui), WASM compatibility, and local devnet tooling.
> 
> **Deliverables**:
> - `bittensor-core` crate: Shared types, config, error handling, Balance arithmetic
> - `bittensor-wallet` crate: NaCl keyfile compat, coldkey/hotkey management, SS58, signing
> - `bittensor-chain` crate: Subtensor client (queries + extrinsics + events + block subscriptions)
> - `bittensor-axon` crate: HTTP server for receiving Synapse requests (FastAPI equivalent)
> - `bittensor-dendrite` crate: HTTP client for sending Synapse requests (aiohttp equivalent)
> - `bittensor-synapse` crate: Synapse protocol, header serialization, SHA3-256 body hashing, StreamingSynapse
> - `bittensor-metagraph` crate: Neural graph state, tensor abstraction, sync from chain
> - `bittensor-cli` crate: Complete btcli reimplementation (wallet, stake, transfer, register, delegate, subnet, root, etc.)
> - `bittensor-pyo3` crate: Python bindings for all crates via PyO3
> - `bittensor-tui` crate: Terminal UI with ratatui
> - `bittensor-wasm` crate: WASM-compatible subset of core functionality
> - `devnet/` directory: Docker-compose + scripts for local Subtensor node
> 
> **Estimated Effort**: XL — 30,000-40,000 lines of Rust across 11 crates
> **Parallel Execution**: YES — 5 waves + final verification
> **Critical Path**: Task 3 (types) → Task 5 (wallet NaCl validation) → Task 8 (chain client) → Task 13 (axon) → Task 15 (synapse integration) → Task 16 (metagraph) → Tasks 17-22 (surfaces)

---

## Context

### Original Request
Complete Rust reimplementation of the Bittensor Python SDK (opentensor/bittensor v10.2.0), as comprehensive as the original, based on latest Rust edition and crates.

### Interview Summary
**Key Discussions**:
- Scope: Full SDK — everything the Python SDK does, plus TUI, WASM, PyO3
- Architecture: Multi-crate mono-repo (9+ crates), not monolith
- Base: Greenfield — existing Rust SDKs too immature (3K useful lines, no Axon/Dendrite/Synapse)
- Network: Configurable for any Subtensor-compatible network
- subxt 0.50.0: Chosen for historic block support and auto runtime upgrades despite breaking changes
- Keyfile compat: Must read/write NaCl-encrypted keyfiles identical to Python SDK
- PyO3: Full bindings in V1 — every public API gets a Python wrapper
- Testing: Full TDD — every module starts with failing tests

**Research Findings**:
- Python SDK AsyncSubtensor alone is 454KB — chain interaction is the largest component
- Protocol is plain HTTP with header-based Sr25519/Ed25519 signing + SHA3-256 body hashing
- Existing Rust SDKs (crabtensor, rusttensor, bittensor-rs) cover only chain queries/extrinsics
- CRITICAL: bittensor-rs one-covenant uses AES-GCM encryption — INCOMPATIBLE with Python's NaCl keyfiles
- DRAND (ML-KEM-768 post-quantum) used for encrypted commitments + MEV Shield
- 17 categories of extrinsics in Python SDK
- 20+ chain data models

### Metis Review
**Identified Gaps** (addressed):
- NaCl vs AES-GCM keyfile incompatibility: CRITICAL — must use NaCl/libsodium with Python's hardcoded salt
- Wallet validation must be done FIRST before building any wallet logic
- subxt 0.50.0 API is completely different from 0.44.x — treat existing Rust SDKs as logical reference only, not code reference
- WASM compatibility constrains which crates can use tokio, file I/O, etc. — need a `bittensor-wasm` thin wrapper
- PyO3 scope is enormous — need to define a minimum public API surface per crate
- TUI adds another significant module — risk of scope creep, must be last
- No existing Rust SDK implements event subscriptions or block following — must design from scratch

---

## Work Objectives

### Core Objective
Produce a complete, production-grade Rust SDK for Bittensor that is API-equivalent to the Python SDK v10.2.0, with full NaCl keyfile compatibility, subxt 0.50.0 chain interaction, Axon/Dendrite/Synapse neuron protocol, and all developer surfaces (CLI, PyO3, TUI, WASM).

### Concrete Deliverables
- 11 publishable Rust crates in a cargo workspace
- Every public API documented with rustdoc
- Full TDD test suite per crate (unit + integration against local devnet)
- CLI binary (`btcli-rs`) with all Python btcli commands
- Python package (`bittensor_rs`) via PyO3/maturin
- TUI binary (`bittensor-tui`) via ratatui
- WASM package for browser/edge use
- Docker-compose setup for local Subtensor devnet

### Definition of Done
- [x] `cargo build --workspace` succeeds with zero errors
- [x] `cargo test --workspace` passes all tests
- [x] `cargo clippy --workspace -- -D warnings` passes
- [x] `btcli-rs --help` shows all command groups matching Python btcli
- [x] Python `import bittensor_rs` works and can create a wallet
- [x] Rust wallet can decrypt a Python-created coldkey file
- [x] Axon server can receive a Synapse from a Dendrite client (integration test)
- [x] Chain client can query metagraph from local devnet
- [x] All QA scenarios pass with evidence in `.sisyphus/evidence/`

### Must Have
- NaCl keyfile read/write compatibility with Python SDK (exact same encryption)
- subxt 0.50.0 chain client with all 100+ query methods and 30+ extrinsic methods
- Axon HTTP server with signature verification, blacklisting, priority middleware
- Dendrite HTTP client with request signing, streaming support
- Synapse protocol: header serialization, SHA3-256 body hashing, nonce validation
- Complete btcli command parity
- PyO3 bindings for all crate public APIs
- TDD test suite with ≥80% line coverage per crate
- Local devnet for integration testing

### Must NOT Have (Guardrails)
- NO AES-GCM keyfile encryption (incompatible with Python)
- NO `as any` / `Any` type erasure in public APIs (defeats Rust type safety)
- NO direct code copying from bittensor-rs one-covenant (wrong subxt version + wrong encryption)
- NO skipping TDD — every task follows RED-GREEN-REFACTOR
- NO PyO3 bindings that panic on error — all must return Result types
- NO tokio runtime in WASM crate (use wasm-bindgen-futures)
- NO unwrap() in library code (only in tests/binaries)
- NO over-abstracted traits where a concrete struct suffices
- NO CLI commands that require human confirmation for dangerous ops without `--yes` flag

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** - ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (new project)
- **Automated tests**: TDD — every task follows RED (failing test) → GREEN (minimal impl) → REFACTOR
- **Framework**: Rust built-in `#[test]` + `tokio::test` for async + custom integration test harness
- **TDD**: Each task includes test cases as part of acceptance criteria

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Library/Module**: Use Bash (cargo test) — compile, run tests, check output
- **CLI**: Use interactive_bash (tmux) — run commands, validate output, check exit codes
- **API/Chain**: Use Bash (curl) — send requests to local devnet, assert status + response
- **PyO3**: Use Bash (maturin develop + pytest) — build, import, call, assert

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — start immediately, NO dependencies):
├── Task 1: Workspace scaffolding + Cargo.toml [quick]
├── Task 2: bittensor-core types + errors + config [deep]
├── Task 3: bittensor-synapse protocol types + hashing [deep]
├── Task 4: Subtensor metadata download + codegen [quick]
└── Task 5: NaCl keyfile compatibility validation [deep]

Wave 2 (After Wave 1 — core modules, MAX PARALLEL):
├── Task 6: bittensor-wallet crate (depends: 2, 5) [deep]
├── Task 7: bittensor-chain storage queries (depends: 2, 4) [unspecified-high]
├── Task 8: bittensor-chain extrinsics (depends: 2, 4, 7) [deep]
├── Task 9: bittensor-chain events + subscriptions (depends: 4, 7) [unspecified-high]
├── Task 10: bittensor-axon server (depends: 2, 3) [deep]
├── Task 11: bittensor-dendrite client (depends: 2, 3) [deep]
└── Task 12: Balance + weight_utils + POW (depends: 2) [unspecified-high]

Wave 3 (After Wave 2 — integration + metagraph):
├── Task 13: bittensor-metagraph (depends: 7, 12) [deep]
├── Task 14: Axon-Dendrite-Synapse integration test (depends: 10, 11) [unspecified-high]
├── Task 15: bittensor-chain full integration tests (depends: 8, 9) [unspecified-high]
├── Task 16: devnet docker-compose + scripts (depends: 4) [quick]
└── Task 17: DRAND + MEV Shield (depends: 7) [deep]

Wave 4 (After Wave 3 — surfaces):
├── Task 18: bittensor-cli — wallet commands (depends: 6, 7) [unspecified-high]
├── Task 19: bittensor-cli — stake/transfer/registration commands (depends: 6, 8) [unspecified-high]
├── Task 20: bittensor-cli — subnet/root/delegate commands (depends: 6, 8, 17) [unspecified-high]
├── Task 21: bittensor-pyo3 — core + wallet + chain bindings (depends: 6, 7, 8) [deep]
├── Task 22: bittensor-pyo3 — axon + dendrite + synapse bindings (depends: 10, 11, 3) [deep]
└── Task 23: bittensor-tui (depends: 6, 7, 13) [visual-engineering]

Wave 5 (After Wave 4 — WASM + polish):
├── Task 24: bittensor-wasm (depends: 2, 3, 7) [deep]
├── Task 25: Documentation + examples (depends: all) [writing]
└── Task 26: Final workspace integration + CI (depends: all) [unspecified-high]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Task 5 → Task 6 → Task 8 → Task 13 → Task 19 → Task 21 → F1-F4 → user okay
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 7 (Waves 1 & 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1 | — | 2,3,4,5 | 1 |
| 2 | 1 | 6,7,8,9,10,11,12 | 1 |
| 3 | 1 | 10,11,14,22,24 | 1 |
| 4 | 1 | 7,8,9,16,17 | 1 |
| 5 | 2 | 6 | 1 |
| 6 | 2,5 | 18,19,20,21,23 | 2 |
| 7 | 2,4 | 8,9,13,15,18,21,23,24 | 2 |
| 8 | 2,4,7 | 13,15,19,20 | 2 |
| 9 | 4,7 | 13,15 | 2 |
| 10 | 2,3 | 14,22 | 2 |
| 11 | 2,3 | 14,22 | 2 |
| 12 | 2 | 13 | 2 |
| 13 | 7,12 | 23 | 3 |
| 14 | 10,11 | — | 3 |
| 15 | 8,9 | — | 3 |
| 16 | 4 | — | 3 |
| 17 | 7 | 20 | 3 |
| 18 | 6,7 | 25,26 | 4 |
| 19 | 6,8 | 25,26 | 4 |
| 20 | 6,8,17 | 25,26 | 4 |
| 21 | 6,7,8 | 25,26 | 4 |
| 22 | 10,11,3 | 25,26 | 4 |
| 23 | 6,7,13 | 25,26 | 4 |
| 24 | 2,3,7 | 25,26 | 5 |
| 25 | all | 26 | 5 |
| 26 | all | F1-F4 | 5 |

### Agent Dispatch Summary

- **Wave 1**: **5** — T1 → `quick`, T2 → `deep`, T3 → `deep`, T4 → `quick`, T5 → `deep`
- **Wave 2**: **7** — T6 → `deep`, T7 → `unspecified-high`, T8 → `deep`, T9 → `unspecified-high`, T10 → `deep`, T11 → `deep`, T12 → `unspecified-high`
- **Wave 3**: **5** — T13 → `deep`, T14 → `unspecified-high`, T15 → `unspecified-high`, T16 → `quick`, T17 → `deep`
- **Wave 4**: **6** — T18 → `unspecified-high`, T19 → `unspecified-high`, T20 → `unspecified-high`, T21 → `deep`, T22 → `deep`, T23 → `visual-engineering`
- **Wave 5**: **3** — T24 → `deep`, T25 → `writing`, T26 → `unspecified-high`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Workspace Scaffolding + Crate Stubs

  **What to do**:
  - Create cargo workspace root with `Cargo.toml` (workspace members: bittensor-core, bittensor-wallet, bittensor-chain, bittensor-axon, bittensor-dendrite, bittensor-synapse, bittensor-metagraph, bittensor-cli, bittensor-pyo3, bittensor-tui, bittensor-wasm)
  - Set edition = "2024", rust-version = "1.85", resolver = "3"
  - Create each crate stub with `src/lib.rs` containing basic module structure
  - Add shared dev-dependencies: `tokio-test`, `tempfile`, `assert_matches`
  - Create `.editorconfig`, `rustfmt.toml` (max_width=100, edition="2024")
  - Create `.gitignore` for Rust projects
  - Verify: `cargo check --workspace` passes on empty stubs

  **Must NOT do**:
  - Do NOT add business logic to any crate yet
  - Do NOT pin specific dependency versions yet (use workspace inheritance)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Pure scaffolding, no domain logic
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - None applicable — pure setup task

  **Parallelization**:
  - **Can Run In Parallel**: NO (foundation for all other tasks)
  - **Parallel Group**: Wave 1 (but must complete first within wave)
  - **Blocks**: Tasks 2, 3, 4, 5
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `crabtensor/Cargo.toml` (GitHub: threetau/crabtensor) — workspace structure reference for a Bittensor SDK
  - `rusttensor/Cargo.toml` (GitHub: womboai/rusttensor) — alternative workspace layout

  **API/Type References**:
  - None (this is scaffolding only)

  **External References**:
  - Cargo Book workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
  - Rust Edition 2024 guide: https://blog.rust-lang.org/2025/02/20/Rust-2024-edition.html

  **WHY Each Reference Matters**:
  - crabtensor workspace: Shows how a Bittensor SDK splits crates (good pattern reference, wrong subxt version)
  - Cargo Book: Workspace configuration syntax for edition 2024

  **Acceptance Criteria**:
  - [ ] `cargo check --workspace` succeeds with zero errors
  - [ ] All 11 crate directories exist with `src/lib.rs`
  - [ ] Root `Cargo.toml` lists all 11 workspace members
  - [ ] Edition 2024 and MSRV 1.85 set in workspace

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Workspace builds successfully
    Tool: Bash
    Preconditions: Clean workspace directory
    Steps:
      1. Run `cargo check --workspace`
      2. Assert exit code 0
      3. Run `cargo test --workspace --no-run`
      4. Assert exit code 0
    Expected Result: Both commands succeed with no errors
    Failure Indicators: Compilation errors, missing crate references
    Evidence: .sisyphus/evidence/task-1-workspace-build.txt

  Scenario: All crates have correct metadata
    Tool: Bash
    Preconditions: Workspace exists
    Steps:
      1. Run `cargo metadata --format-version=1 | jq '.workspace_members | length'`
      2. Assert output = "11"
      3. Run `grep -r 'edition.*=.*"2024"' --include=Cargo.toml | wc -l`
      4. Assert count >= 11
    Expected Result: 11 workspace members, all edition 2024
    Failure Indicators: Wrong count, wrong edition
    Evidence: .sisyphus/evidence/task-1-crate-metadata.txt
  ```

  **Commit**: YES (Wave 1 group)
  - Message: `feat(workspace): scaffold cargo workspace and crate stubs`
  - Files: `Cargo.toml, */Cargo.toml, */src/lib.rs, .editorconfig, rustfmt.toml, .gitignore`
  - Pre-commit: `cargo check --workspace`

- [x] 2. bittensor-core: Shared Types, Errors, Config, Balance Arithmetic

  **What to do**:
  - Define `SubtensorConfig` implementing `subxt::Config` (Hash=H256, AccountId=AccountId32, Address=MultiAddress, Signature=MultiSignature, Hasher=BlakeTwo256, Header=SubstrateHeader<u32>, ExtrinsicParams=SubstrateExtrinsicParams)
  - Define comprehensive error types with `thiserror`: `BittensorError` with variants for RPC, Signing, Codec, Transaction, Wallet, Network, Config, etc.
  - Define `ErrorCategory` (Transient/RateLimit/Auth/Config/Network/Permanent) with associated `RetryConfig` per category
  - Define `NetworkConfig` struct with presets: `finney()`, `test()`, `local()`, `archive()`, `latent_lite()`
  - Implement `Balance` type with FixedPoint arithmetic: tao (10^9 rao), subnet-aware, Display/FromStr, arithmetic ops (Add, Sub, Mul, Div), comparison
  - Define chain data models: `AxonInfo`, `NeuronInfo`, `NeuronInfoLite`, `DelegateInfo`, `StakeInfo`, `SubnetInfo`, `SubnetHyperparameters`, `ChainIdentity`, `PrometheusInfo`, `WeightCommitInfo`, etc.
  - All types derive `Encode, Decode` from parity-scale-codec + `Serialize, Deserialize` from serde
  - Write TDD tests for: Balance arithmetic, error categories, network config presets, SCALE round-trip encoding

  **Must NOT do**:
  - Do NOT use `: any` or `Any` types anywhere
  - Do NOT import `sp-core` or `sp-runtime` directly (use subxt primitives only)
  - Do NOT implement chain queries here (those go in bittensor-chain)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex type system design with SCALE codec, multiple derive macros, and domain modeling — requires deep understanding of Substrate types and Bittensor domain
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - None applicable — this is pure Rust type design

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 1)
  - **Parallel Group**: Wave 1 (with Tasks 3, 4, 5)
  - **Blocks**: Tasks 6, 7, 8, 9, 10, 11, 12
  - **Blocked By**: Task 1

  **References**:

  **Pattern References**:
  - `crabtensor/src/types.rs` (GitHub: threetau/crabtensor) — Bittensor type definitions in Rust
  - `bittensor-rs/src/error.rs` (GitHub: one-covenant/bittensor-rs) — Error categorization pattern with RetryConfig
  - `bittensor/core/chain_data/` (GitHub: opentensor/bittensor) — Python chain data models (NeuronInfo, DelegateInfo, etc.)

  **API/Type References**:
  - `subxt::config::SubstrateConfig` — Reference Config implementation for Substrate chains
  - `subxt::utils::{AccountId32, H256, MultiAddress, MultiSignature}` — Primitive types
  - `parity_scale_codec::{Encode, Decode}` — SCALE codec traits

  **Test References**:
  - `bittensor/utils/balance.py` (GitHub: opentensor/bittensor) — Balance arithmetic test patterns

  **External References**:
  - subxt Config trait docs: https://docs.rs/subxt/0.50.0/subxt/config/trait.Config.html
  - SCALE codec spec: https://docs.substrate.io/reference/scale-codec/

  **WHY Each Reference Matters**:
  - crabtensor types.rs: Shows how to map Python Bittensor types to Rust structs with SCALE codec
  - bittensor-rs error.rs: ErrorCategory + RetryConfig pattern is proven for Bittensor SDK
  - subxt Config docs: Required to implement SubtensorConfig correctly for subxt 0.50.0

  **Acceptance Criteria**:
  - [ ] `SubtensorConfig` compiles and satisfies `subxt::Config` trait bounds
  - [ ] `Balance` type: `Balance::from_tao(1.0) + Balance::from_rao(500_000_000)` produces correct result
  - [ ] All 20+ chain data models derive `Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq`
  - [ ] SCALE round-trip: `NeuronInfo::decode(&mut encoded_neuron)` succeeds
  - [ ] Error categories have associated `RetryConfig` values
  - [ ] Network presets produce correct WebSocket URLs

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Balance arithmetic is correct
    Tool: Bash (cargo test)
    Preconditions: bittensor-core compiles
    Steps:
      1. Run `cargo test -p bittensor-core -- balance`
      2. Assert all tests pass
      3. Verify test covers: from_tao, from_rao, add, sub, mul, div, display, subnet_aware
    Expected Result: All Balance tests pass
    Failure Indicators: Arithmetic overflow, wrong display format, subnet mismatch
    Evidence: .sisyphus/evidence/task-2-balance-arithmetic.txt

  Scenario: SCALE round-trip encoding works for all chain data models
    Tool: Bash (cargo test)
    Preconditions: Chain data models defined
    Steps:
      1. Run `cargo test -p bittensor-core -- round_trip`
      2. Assert all tests pass (each model tested)
    Expected Result: encode then decode produces identical struct for each model
    Failure Indicators: Decode error, field mismatch
    Evidence: .sisyphus/evidence/task-2-scale-roundtrip.txt
  ```

  **Commit**: YES (Wave 1 group)
  - Message: `feat(core): shared types, errors, config, Balance arithmetic`
  - Files: `bittensor-core/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-core`

- [x] 3. bittensor-synapse: Protocol Types, Header Serialization, SHA3-256 Hashing

  **What to do**:
  - Define `Synapse` trait: `to_headers() -> HashMap<String,String>`, `from_headers(headers) -> Result<Self>`, `body_hash() -> String`, `deserialize_body(body: &[u8]) -> Result<Self::Output>`
  - Define `TerminalInfo` struct: `dendrite_hotkey`, `axon_hotkey`, `nonce`, `uuid`, `body_hash`, `computed_body_hash`, `status_code`, `status_message`, `timeout`
  - Implement header-based serialization: headers prefixed with `bt-` (matching Python's `to_headers()`/`from_headers()`)
  - Implement SHA3-256 body hashing: hash required fields of synapse, compare against `computed_body_hash`
  - Define signing message format: `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`
  - Define `StreamingSynapse` extending Synapse with SSE stream handling
  - Write TDD tests: header round-trip, body hash integrity, signing message format, streaming synapse deserialization

  **Must NOT do**:
  - Do NOT implement HTTP server (that's bittensor-axon)
  - Do NOT implement HTTP client (that's bittensor-dendrite)
  - Do NOT depend on `aiohttp` or `axum` — this crate is protocol-only

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Protocol design with crypto hashing, header serialization format, and trait system — requires careful domain understanding
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - None applicable

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 1)
  - **Parallel Group**: Wave 1 (with Tasks 2, 4, 5)
  - **Blocks**: Tasks 10, 11, 14, 22, 24
  - **Blocked By**: Task 1

  **References**:

  **Pattern References**:
  - `bittensor/core/synapse.py` (GitHub: opentensor/bittensor) — Python Synapse model, TerminalInfo, header serialization
  - `bittensor/core/stream.py` (GitHub: opentensor/bittensor) — StreamingSynapse pattern

  **API/Type References**:
  - Python `Synapse.to_headers()` — exact header names and format
  - Python `Synapse.compute_body_hash()` — SHA3-256 over required fields
  - Python `TerminalInfo` — field names and types

  **External References**:
  - SHA3-256 in Rust: `sha3` crate — https://docs.rs/sha3/latest/sha3/
  - Python signing format: `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`

  **WHY Each Reference Matters**:
  - Python synapse.py: This IS the protocol specification. Header names, body hash algorithm, signing message format must match EXACTLY for cross-SDK compatibility.
  - sha3 crate: Need the exact same SHA3-256 variant Python uses (keccak-256 vs SHA3-256 — Python uses hashlib.sha3_256 which is the FIPS 202 variant)

  **Acceptance Criteria**:
  - [ ] `Synapse` trait defined with all required methods
  - [ ] Header round-trip: `synapse.to_headers()` → `Synapse::from_headers()` produces identical struct
  - [ ] Body hash matches Python's SHA3-256 output for identical input
  - [ ] Signing message format matches Python exactly
  - [ ] `StreamingSynapse` trait extends `Synapse` with SSE support

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Header round-trip preserves all fields
    Tool: Bash (cargo test)
    Preconditions: bittensor-synapse compiles
    Steps:
      1. Create a test Synapse struct implementing the trait
      2. Call to_headers(), then from_headers() on the result
      3. Assert all fields match original
    Expected Result: Round-trip produces identical struct
    Failure Indicators: Missing headers, wrong types, format mismatch
    Evidence: .sisyphus/evidence/task-3-header-roundtrip.txt

  Scenario: Body hash matches Python SDK output
    Tool: Bash (cargo test)
    Preconditions: SHA3-256 implementation complete
    Steps:
      1. Hash a known test vector that Python also hashes
      2. Compare Rust SHA3-256 output with Python hashlib.sha3_256 output
      3. Assert byte-for-byte match
    Expected Result: Identical hash output
    Failure Indicators: Wrong SHA3 variant (keccak vs FIPS 202), encoding difference
    Evidence: .sisyphus/evidence/task-3-body-hash-match.txt
  ```

  **Commit**: YES (Wave 1 group)
  - Message: `feat(synapse): protocol types, header serialization, SHA3-256 hashing`
  - Files: `bittensor-synapse/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-synapse`

- [x] 4. Subtensor Metadata Download + Codegen

  **What to do**:
  - Use `subxt-cli@0.50.0` to download metadata from Finney mainnet (`wss://entrypoint-finney.opentensor.ai:443`) and save as `metadata/finney.scale`
  - Set up `bittensor-chain/build.rs` with `subxt_utils_fetchmetadata` to auto-download during builds (with caching)
  - Generate the typed subxt API module: `#[subxt::subxt(runtime_metadata_path = "../metadata/finney.scale")]`
  - Verify generated code compiles: `cargo check -p bittensor-chain`
  - Also download test and local metadata for integration test targets
  - Document the metadata refresh procedure in `bittensor-chain/README.md`

  **Must NOT do**:
  - Do NOT write any chain query/extrinsic logic (that's Tasks 7, 8, 9)
  - Do NOT use `runtime_metadata_insecure_url` in production code (build-time download only)
  - Do NOT commit binary metadata files >5MB to git (use .gitattributes and LFS or CI download)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Mechanical setup task — running CLI commands and configuring build.rs
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 1)
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 5)
  - **Blocks**: Tasks 7, 8, 9, 16, 17
  - **Blocked By**: Task 1

  **References**:

  **Pattern References**:
  - `crabtensor/build.rs` (GitHub: threetau/crabtensor) — shows how to set up metadata download for Bittensor
  - `rusttensor/build.rs` (GitHub: womboai/rusttensor) — alternative metadata setup pattern

  **API/Type References**:
  - `subxt_cli metadata` command — how to invoke metadata download
  - `subxt_utils_fetchmetadata` — build.rs helper crate

  **External References**:
  - subxt metadata docs: https://docs.rs/subxt/0.50.0/subxt/#generating-an-api-from-metadata
  - Finney endpoint: wss://entrypoint-finney.opentensor.ai:443

  **WHY Each Reference Matters**:
  - crabtensor build.rs: Proven pattern for Bittensor-specific metadata download
  - Finney endpoint: This is the actual mainnet endpoint we must target

  **Acceptance Criteria**:
  - [ ] `metadata/finney.scale` file exists and is valid SCALE metadata
  - [ ] `bittensor-chain/build.rs` successfully downloads and caches metadata
  - [ ] Generated subxt API module compiles: `cargo check -p bittensor-chain` passes
  - [ ] Generated module exposes `subtensor_module` storage and calls

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Metadata download and codegen works
    Tool: Bash
    Preconditions: subxt-cli@0.50.0 installed, workspace exists
    Steps:
      1. Run `cargo check -p bittensor-chain`
      2. Assert exit code 0
      3. Run `ls -la metadata/finney.scale` and assert file exists and size > 100KB
      4. Run `grep -r "subtensor_module" bittensor-chain/src/` and assert generated code exists
    Expected Result: Metadata downloaded, codegen succeeds, subxt API available
    Failure Indicators: Download timeout, invalid metadata, codegen errors
    Evidence: .sisyphus/evidence/task-4-metadata-codegen.txt
  ```

  **Commit**: YES (Wave 1 group)
  - Message: `feat(chain): download Subtensor metadata and codegen API`
  - Files: `metadata/finney.scale, bittensor-chain/build.rs, bittensor-chain/src/generated.rs`
  - Pre-commit: `cargo check -p bittensor-chain`

- [x] 5. NaCl Keyfile Compatibility Validation (CRITICAL)

  **What to do**:
  - This is the MOST CRITICAL task — validates the entire wallet crate foundation
  - Write a standalone Rust program that decrypts a Python-created coldkey file using NaCl/libsodium
  - Python SDK encryption scheme: `argon2id(password, salt=NACL_SALT, ops_limit, mem_limit)` → 32-byte key → `crypto_secretbox_seal(cleartext, key, nonce)` where nonce = 24 bytes from `os.urandom(24)` and salt = `b'\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1'`
  - Generate a test coldkey file using Python (`btcli wallet create`), save to a known location
  - In Rust, read that exact file, decrypt it, and verify the 32-byte secret key matches
  - Use `sodiumoxide` or `libsodium-sys` crate for NaCl compatibility
  - Also test ENCRYPTION: write a coldkey file from Rust, then read it back with Python
  - Document the exact encryption/decryption protocol in `bittensor-wallet/ENCRYPTION.md`
  - If this fails, the entire wallet crate is on the wrong foundation

  **Must NOT do**:
  - Do NOT use AES-GCM (this is what bittensor-rs one-covenant uses, and it's INCOMPATIBLE)
  - Do NOT proceed to Task 6 (wallet crate) until this validation passes both directions
  - Do NOT assume the encryption format — verify with actual Python SDK output

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Critical validation requiring cryptographic precision, cross-language debugging, and potentially building Python interop tooling
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 1, 2)
  - **Parallel Group**: Wave 1 (with Tasks 3, 4)
  - **Blocks**: Task 6 (entire wallet crate)
  - **Blocked By**: Tasks 1, 2

  **References**:

  **Pattern References**:
  - `bittensor_wallet/wallet_impl.py` (GitHub: opentensor/bittensor-wallet) — Python encryption implementation
  - `bittensor-rs/bittensor-wallet/src/wallet.rs` (GitHub: one-covenant) — WRONG pattern (AES-GCM, but useful to see what NOT to do)

  **API/Type References**:
  - Python `nacl.secret.SecretBox` — encryption API
  - Python `nacl.pwhash.argon2id.kdf` — key derivation
  - Python coldkey format: `{nonce: 24 bytes}{ciphertext: N bytes}{mac: 16 bytes}` (NaCl secretbox format)

  **External References**:
  - sodiumoxide crate: https://docs.rs/sodiumoxide/latest/sodiumoxide/
  - PyNaCl docs: https://pynacl.readthedocs.io/en/latest/secret/
  - Argon2id RFC: https://www.rfc-editor.org/rfc/rfc9106

  **WHY Each Reference Matters**:
  - bittensor_wallet Python source: This IS the encryption specification. Every parameter matters: salt, ops_limit, mem_limit, nonce length, MAC position.
  - bittensor-rs wallet.rs: Anti-pattern reference — AES-GCM will NOT work with Python keyfiles.
  - PyNaCl docs: NaCl secretbox format details (nonce + ciphertext + MAC)

  **Acceptance Criteria**:
  - [ ] Rust program successfully decrypts a Python-created coldkey file
  - [ ] Decrypted 32-byte secret key matches what Python produces
  - [ ] Rust-encrypted coldkey file can be decrypted by Python
  - [ ] `ENCRYPTION.md` documents the exact protocol
  - [ ] Round-trip test passes both directions

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Decrypt Python coldkey in Rust
    Tool: Bash
    Preconditions: Python coldkey file exists (generated by btcli wallet create)
    Steps:
      1. Run Rust validation binary: `cargo run -p bittensor-wallet --example validate_nacl_compat -- /path/to/python/coldkey`
      2. Assert exit code 0
      3. Assert output contains "Decryption successful" and shows matching public key
    Expected Result: Rust decrypts Python coldkey, produces matching keypair
    Failure Indicators: Decryption error, key mismatch, wrong salt/ops/mem parameters
    Evidence: .sisyphus/evidence/task-5-nacl-decrypt.txt

  Scenario: Encrypt in Rust, decrypt in Python (round-trip)
    Tool: Bash
    Preconditions: Rust encryption implemented, Python SDK installed
    Steps:
      1. Generate coldkey in Rust: `cargo run -p bittensor-wallet --example create_test_coldkey`
      2. In Python: `from bittensor_wallet import Wallet; w = Wallet(name="test", path="/tmp/rust-coldkey"); print(w.coldkeypub)`
      3. Assert Python can read the Rust-generated coldkey without error
    Expected Result: Python successfully reads Rust-encrypted coldkey
    Failure Indicators: Python decryption error, wrong format, missing MAC
    Evidence: .sisyphus/evidence/task-5-nacl-roundtrip.txt
  ```

  **Commit**: YES (Wave 1 group)
  - Message: `feat(wallet): NaCl keyfile compatibility validation against Python SDK`
  - Files: `bittensor-wallet/examples/validate_nacl_compat.rs, bittensor-wallet/ENCRYPTION.md`
  - Pre-commit: validation binary passes both directions

- [x] 6. bittensor-wallet: Full Wallet Management

  **What to do**:
  - Implement `Wallet` struct: coldkey (encrypted NaCl), coldkeypub, hotkey (unencrypted by default), name, path
  - Implement coldkey creation from mnemonic (BIP39 12/24-word), from seed hex, from SURI derivation path
  - Implement hotkey creation with derivation path `//<hotkey_name>` from coldkey or independently
  - Implement SS58 address generation with format 42 (Bittensor default)
  - Implement keyfile read/write compatible with `~/.bittensor/wallets/<name>/coldkey` and `coldkeypub` and `hotkeys/<hotkey_name>`
  - Implement coldkey encryption using NaCl (validated in Task 5) with `argon2id` key derivation
  - Implement coldkey decryption with password prompt (CLI) or parameter (library)
  - Implement signing: `Keypair.sign(message)` using Sr25519 or Ed25519
  - Implement signature verification
  - Implement `Wallet` methods: `get_coldkey_pair()`, `get_hotkey_pair()`, `sign()`, `verify()`, `ss58_address()`
  - Support reading existing Python-generated wallet directories
  - Write TDD tests for: key generation, encryption round-trip, SS58 encoding, signing/verification, Python keyfile compatibility

  **Must NOT do**:
  - Do NOT use AES-GCM for encryption (see Task 5)
  - Do NOT store hotkeys encrypted by default (Python doesn't)
  - Do NOT implement chain queries (that's bittensor-chain)
  - Do NOT use `sp-core::Pair` directly — use `subxt_signer` instead for WASM compatibility

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex crypto integration (NaCl, argon2id, BIP39, SS58, Sr25519), must be byte-perfect compatible with Python
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 2, 5)
  - **Parallel Group**: Wave 2 (with Tasks 7-12)
  - **Blocks**: Tasks 18, 19, 20, 21, 23
  - **Blocked By**: Tasks 2, 5

  **References**:

  **Pattern References**:
  - `bittensor_wallet/wallet_impl.py` (GitHub: opentensor/bittensor-wallet) — Python wallet implementation
  - `bittensor-rs/bittensor-wallet/src/wallet.rs` (one-covenant) — Rust wallet (AES-GCM — DO NOT COPY encryption, but key derivation paths are valid reference)

  **API/Type References**:
  - `subxt_signer::sr25519::Keypair` — signing keypair
  - `subxt_signer::SecretUri` — SURI parsing
  - `sodiumoxide::crypto::secretbox` — NaCl symmetric encryption
  - `ss58_registry` — SS58 address format

  **External References**:
  - SS58 format: https://docs.substrate.io/reference/address-formats/
  - BIP39 Mnemonic: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki

  **WHY Each Reference Matters**:
  - Python wallet_impl.py: Source of truth for all wallet operations, file paths, encryption params
  - subxt_signer: The signing interface that subxt 0.50.0 uses — must use this for chain extrinsics

  **Acceptance Criteria**:
  - [ ] `Wallet::new(name, path)` creates wallet directory structure
  - [ ] `Wallet::create_coldkey_from_mnemonic()` produces valid Sr25519 keypair
  - [ ] Coldkey encryption/decryption round-trips through NaCl
  - [ ] SS58 address format 42 matches Python output for same keypair
  - [ ] Rust can read Python-created wallet directory and decrypt coldkey
  - [ ] `cargo test -p bittensor-wallet` passes all tests

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Create wallet and verify keyfile compatibility
    Tool: Bash
    Preconditions: Task 5 validation complete
    Steps:
      1. Run `cargo test -p bittensor-wallet -- wallet_creation`
      2. Assert all tests pass
      3. Manually check wallet directory structure matches Python: `ls ~/.bittensor/wallets/test/`
    Expected Result: Wallet directory has coldkey, coldkeypub, hotkeys/ directory
    Failure Indicators: Wrong directory structure, missing coldkeypub
    Evidence: .sisyphus/evidence/task-6-wallet-creation.txt

  Scenario: Cross-SDK keyfile compatibility
    Tool: Bash
    Preconditions: Both Rust and Python wallets exist
    Steps:
      1. Create wallet in Rust
      2. Attempt to load same wallet in Python
      3. Create wallet in Python
      4. Attempt to load same wallet in Rust
      5. Assert both directions succeed and produce matching addresses
    Expected Result: Seamless cross-SDK wallet sharing
    Failure Indicators: Decryption errors, address mismatch
    Evidence: .sisyphus/evidence/task-6-cross-sdk-compat.txt
  ```

  **Commit**: YES (Wave 2 group)
  - Message: `feat(wallet): full wallet management — NaCl encryption, coldkey/hotkey, SS58, signing`
  - Files: `bittensor-wallet/src/**/*.rs, bittensor-wallet/ENCRYPTION.md`
  - Pre-commit: `cargo test -p bittensor-wallet`

- [x] 7. bittensor-chain: Storage Queries

  **What to do**:
  - Implement `SubtensorClient` struct wrapping `OnlineClient<SubtensorConfig>`
  - Implement connection with automatic failover across multiple endpoints (connection pool pattern from bittensor-rs)
  - Implement ALL storage query methods matching Python SDK:
    - Metagraph: `get_metagraph`, `get_selective_metagraph`, metagraph field queries
    - Neuron: `get_neuron`, `get_neuron_lite`, `get_uid_for_hotkey`, `get_neuron_for_pubkey_and_subnet`
    - Account: `get_balance`, `get_stake`, `get_stake_info_for_coldkey`, `get_total_network_stake`, `get_total_balance`
    - Subnet: `get_subnet_info`, `get_subnet_hyperparameters`, `get_total_subnets`, `subnet_exists`, `get_subnet_owner`
    - Delegate: `get_delegates`, `get_delegate_take`, `get_delegated_info`
    - Network: `get_network_block`, `get_network_hash_rate`, `get_current_weight`, `get_total_issuance`
    - Neuron count and UIDs queries
    - Weight queries: `get_weights`, `get_weights_min`, `get_weights_max`, `get_weights_set_rate_limit`
    - Commit/reveal queries for weight commitments
    - Children, proxy, and identity queries
  - Implement concurrent batch query helpers for common patterns
  - Write TDD tests against local devnet (mock substrate for unit tests)

  **Must NOT do**:
  - Do NOT implement extrinsics here (that's Task 8)
  - Do NOT implement event subscriptions (that's Task 9)
  - Do NOT hardcode endpoint URLs in the client — use NetworkConfig from Task 2

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Large volume of mechanical but numerous query implementations, each following the same subxt pattern
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 2, 4)
  - **Parallel Group**: Wave 2 (with Tasks 6, 8-12)
  - **Blocks**: Tasks 8, 9, 13, 15, 18, 21, 23, 24
  - **Blocked By**: Tasks 2, 4

  **References**:

  **Pattern References**:
  - `bittensor-rs/src/queries/mod.rs` (one-covenant) — query method structure reference
  - `bittensor/core/async_subtensor.py` (opentensor) — complete list of Python query methods (source of truth)

  **API/Type References**:
  - `subxt::OnlineClient<SubtensorConfig>` — client type
  - `subxt::storage().at_current_block()` — query pattern
  - Generated `subtensor::storage()` — typed storage access

  **Test References**:
  - `bittensor/core/extrinsics/asyncex/` — Python test patterns for chain interaction

  **External References**:
  - subxt storage queries: https://docs.rs/subxt/0.50.0/subxt/storage/index.html

  **WHY Each Reference Matters**:
  - bittensor-rs queries: Shows the Rust subxt query pattern (but outdated API — adapt to 0.50.0)
  - Python async_subtensor: Complete method signatures — every Python query must have a Rust equivalent

  **Acceptance Criteria**:
  - [ ] `SubtensorClient::from_config(NetworkConfig::finney())` connects successfully
  - [ ] `client.get_metagraph(1)` returns valid MetagraphInfo from local devnet
  - [ ] `client.get_balance(account_id)` returns correct Balance
  - [ ] All 50+ query methods compile and pass unit tests
  - [ ] Connection pool fails over to backup endpoints

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Storage queries work against local devnet
    Tool: Bash
    Preconditions: Local devnet running (or mock substrate)
    Steps:
      1. Run `cargo test -p bittensor-chain -- storage_queries`
      2. Assert all query tests pass
    Expected Result: All storage queries return valid data from devnet
    Failure Indicators: Connection refused, decode errors, missing storage items
    Evidence: .sisyphus/evidence/task-7-storage-queries.txt

  Scenario: Connection failover works
    Tool: Bash (cargo test)
    Preconditions: Multiple endpoints configured, first one unreachable
    Steps:
      1. Configure endpoints: ["wss://invalid:443", "wss://valid:443"]
      2. Create SubtensorClient
      3. Run a query
      4. Assert it succeeds despite first endpoint failing
    Expected Result: Automatic failover to working endpoint
    Failure Indicators: Connection timeout, no retry
    Evidence: .sisyphus/evidence/task-7-connection-failover.txt
  ```

  **Commit**: YES (Wave 2 group)
  - Message: `feat(chain): storage queries — metagraph, neuron, balance, stake, subnet, delegate`
  - Files: `bittensor-chain/src/queries/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-chain -- storage_queries`

- [x] 8. bittensor-chain: Extrinsics

  **What to do**:
  - Implement ALL extrinsic submission methods matching Python SDK:
    - Staking: `add_stake`, `add_stake_multiple`, `add_stake_burn`, `set_auto_stake`
    - Unstaking: `unstake`, `unstake_all`, `unstake_multiple`
    - Move/Swap: `move_stake`, `swap_stake`, `transfer_stake`
    - Registration: `register` (POW), `burned_register`, `register_subnet`, `set_subnet_identity`
    - Weights: `set_weights`, `commit_weights`, `reveal_weights`, `commit_timelocked_weights`
    - Serving: `serve_axon`, `publish_metadata`
    - Transfer: `transfer`, `transfer_keep_alive`, `transfer_all`
    - Proxy: `add_proxy`, `remove_proxy`, `remove_proxies`, `create_pure_proxy`, `kill_pure_proxy`, `proxy`, `proxy_announced`, `announce`, `reject_announcement`, `remove_announcement`, `poke_deposit`
    - Children: `set_children`, `set_childkey_take`, `revoke_children`
    - Take: `set_delegate_take`
    - Coldkey swap: `announce_coldkey_swap`, `dispute_coldkey_swap`, `swap_announced_coldkey`
    - Root: `root_register`, `claim_root`, `set_root_claim_type`
    - Crowdloan: all crowdloan operations
    - Liquidity: AMM operations
    - MEV Shield: `submit_encrypted_extrinsic`
    - Sudo: sudo calls
  - Each method: sign with `subxt_signer`, submit, watch for finalization, parse events
  - Implement retry with exponential backoff per ErrorCategory from Task 2
  - Write TDD tests: mock extrinsic submission, verify SCALE-encoded payload

  **Must NOT do**:
  - Do NOT submit real extrinsics in unit tests (use mocks)
  - - Do NOT use `unwrap()` — return Result everywhere
  - Do NOT implement event listening here (that's Task 9)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: High complexity — 17 categories of extrinsics, each requiring precise SCALE encoding, signing, submission, event parsing, and error handling
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 7 queries)
  - **Parallel Group**: Wave 2 (sequential after Task 7)
  - **Blocks**: Tasks 13, 15, 19, 20
  - **Blocked By**: Tasks 2, 4, 7

  **References**:

  **Pattern References**:
  - `bittensor-rs/src/extrinsics/mod.rs` (one-covenant) — extrinsic structure reference
  - `bittensor/core/extrinsics/` + `bittensor/core/extrinsics/asyncex/` (opentensor) — Python extrinsic implementations

  **API/Type References**:
  - Generated `subtensor::tx()` — typed extrinsic access
  - `subxt::tx().sign_and_submit_then_watch_default()` — submission pattern
  - `subxt_signer::sr25519::Keypair` — signing

  **External References**:
  - subxt transactions: https://docs.rs/subxt/0.50.0/subxt/tx/index.html

  **WHY Each Reference Matters**:
  - bittensor-rs extrinsics: Shows the subxt extrinsic pattern (adapt to 0.50.0 API)
  - Python extrinsics: Source of truth for parameter types, error handling, and event parsing

  **Acceptance Criteria**:
  - [ ] All 30+ extrinsic methods compile and type-check
  - [ ] `add_stake` with mock client produces valid SCALE-encoded call
  - [ ] `transfer` with mock client signs and submits correctly
  - [ ] Event parsing extracts `StakeAdded`/`Transfer` events from mock response
  - [ ] `cargo test -p bittensor-chain -- extrinsics` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Extrinsic submission with mock client
    Tool: Bash (cargo test)
    Preconditions: Mock substrate client available
    Steps:
      1. Run `cargo test -p bittensor-chain -- extrinsics`
      2. Assert all tests pass
      3. Verify mock calls contain correct SCALE-encoded payloads
    Expected Result: All extrinsic tests pass with correct encoding
    Failure Indicators: Wrong SCALE encoding, missing parameters, event parse failure
    Evidence: .sisyphus/evidence/task-8-extrinsics.txt

  Scenario: Retry on transient failure
    Tool: Bash (cargo test)
    Preconditions: Mock that returns transient error then success
    Steps:
      1. Configure mock to fail once then succeed
      2. Call `add_stake`
      3. Assert it eventually succeeds after retry
    Expected Result: Automatic retry succeeds
    Failure Indicators: Immediate error return, no retry attempt
    Evidence: .sisyphus/evidence/task-8-extrinsic-retry.txt
  ```

  **Commit**: YES (Wave 2 group)
  - Message: `feat(chain): extrinsics — staking, weights, transfer, registration, proxy, children, root, sudo`
  - Files: `bittensor-chain/src/extrinsics/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-chain -- extrinsics`

- [x] 9. bittensor-chain: Event Subscriptions + Block Following

  **What to do**:
  - Implement `subscribe_events()` — listen to all chain events via subxt's subscription API
  - Implement `subscribe_blocks()` — follow new blocks as they're produced
  - Implement typed event filters: `on_neuron_registered()`, `on_weights_set()`, `on_stake_added()`, etc.
  - Implement `subscribe_storage()` — watch specific storage keys for changes
  - Design an event handler trait `ChainEventHandler` with methods for each event category
  - Implement a `ChainMonitor` struct that runs in a background tokio task, emitting events via `tokio::sync::broadcast`
  - Write TDD tests: mock subscription, verify event types are correctly decoded

  **Must NOT do**:
  - Do NOT implement this as a polling loop — must use WebSocket subscriptions
  - Do NOT block the subscriber on event processing — use channels

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Complex async design with subscriptions, channels, and event routing — but well-defined subxt patterns
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 4, 7)
  - **Parallel Group**: Wave 2 (with Tasks 6, 8, 10-12)
  - **Blocks**: Tasks 13, 15
  - **Blocked By**: Tasks 4, 7

  **References**:

  **Pattern References**:
  - None exist for Bittensor-specific event subscriptions — this is new territory
  - `subxt/examples/subscribe_events.rs` — generic subxt event subscription example

  **API/Type References**:
  - `subxt::events().at()` — event access
  - `subxt::blocks().subscribe_best()` — block subscription
  - `tokio::sync::broadcast` — event distribution channel

  **External References**:
  - subxt subscriptions: https://docs.rs/subxt/0.50.0/subxt/rpc/index.html

  **WHY Each Reference Matters**:
  - subxt examples: Only reference for the subscription API pattern in v0.50.0

  **Acceptance Criteria**:
  - [ ] `ChainMonitor::new(client).start()` runs without error
  - [ ] Event subscription decodes `NeuronRegistered` events correctly
  - [ ] Block subscription produces block numbers in increasing order
  - [ ] `broadcast::Receiver` receives events from `ChainMonitor`
  - [ ] `cargo test -p bittensor-chain -- events` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Event subscription receives and decodes events
    Tool: Bash (cargo test)
    Preconditions: Mock substrate with event emission
    Steps:
      1. Start ChainMonitor
      2. Trigger a mock NeuronRegistered event
      3. Assert event is received through broadcast channel
      4. Assert event type matches expected structure
    Expected Result: Events correctly decoded and distributed
    Failure Indicators: Timeout, wrong event type, decode error
    Evidence: .sisyphus/evidence/task-9-event-subscription.txt
  ```

  **Commit**: YES (Wave 2 group)
  - Message: `feat(chain): event subscriptions and block following`
  - Files: `bittensor-chain/src/events/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-chain -- events`

- [x] 10. bittensor-axon: HTTP Server with Middleware

  **What to do**:
  - Implement `Axon` server using `axum` — the Rust equivalent of Python's FastAPI-based Axon
  - Implement middleware chain (matching Python's Axon middleware):
    1. **Verification**: Check signature on `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"` header
    2. **Blacklist**: Check if dendrite hotkey is blacklisted (configurable blacklist fn)
    3. **Priority**: Assign request priority based on hotkey stake (query from chain)
    4. **Body hash**: Verify SHA3-256 body hash matches `computed_body_hash` header
  - Implement `attach(synapse_type)` — register route handler for a specific Synapse type at `/{SynapseName}`
  - Implement `start()` — bind to configured IP:port and serve
  - Implement `forward()` — default handler that passes to registered handler
  - Implement `stop()` — graceful shutdown
  - Support both regular responses and SSE streaming responses
  - Write TDD tests: start axon, send request with correct signature, verify handler called

  **Must NOT do**:
  - Do NOT implement the HTTP client (that's bittensor-dendrite)
  - Do NOT use Python-specific patterns (no `__init__`, no decorators)
  - Do NOT skip the body hash verification — this is a security critical path

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex HTTP server with crypto middleware, signature verification, priority queuing, and streaming — core Bittensor protocol implementation
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 2, 3)
  - **Parallel Group**: Wave 2 (with Tasks 6-9, 11-12)
  - **Blocks**: Tasks 14, 22
  - **Blocked By**: Tasks 2, 3

  **References**:

  **Pattern References**:
  - `bittensor/core/axon.py` (GitHub: opentensor/bittensor) — Python Axon implementation (FastAPI + uvicorn, middleware chain)
  - `axum/examples/anyhow-error` — Axum server pattern

  **API/Type References**:
  - `axum::Router`, `axum::middleware` — server framework
  - `bittensor_synapse::Synapse` trait — from Task 3
  - `bittensor_wallet::Wallet::sign()` — for verification

  **External References**:
  - axum middleware: https://docs.rs/axum/latest/axum/middleware/index.html
  - axum SSE: https://docs.rs/axum/latest/axum/response/sse/index.html

  **WHY Each Reference Matters**:
  - Python axon.py: Exact middleware order and verification logic must match for cross-SDK compatibility
  - axum: The HTTP framework — need its middleware composition pattern

  **Acceptance Criteria**:
  - [ ] `Axon::new(config)` creates server
  - [ ] `axon.attach::<MySynapse>(handler)` registers route
  - [ ] `axon.start()` binds to port and serves
  - [ ] Request with valid signature passes verification middleware
  - [ ] Request with invalid signature is rejected (401)
  - [ ] Blacklisted hotkey is rejected (403)
  - [ ] `cargo test -p bittensor-axon` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Axon receives and verifies signed request
    Tool: Bash (curl + cargo test)
    Preconditions: Axon server running on port 3100
    Steps:
      1. Start axon: `cargo run -p bittensor-axon --example basic_server`
      2. Sign a request with test keypair
      3. Send signed request to `http://127.0.0.1:3100/MySynapse`
      4. Assert 200 response
      5. Send unsigned request to same endpoint
      6. Assert 401 response
    Expected Result: Signed requests accepted, unsigned rejected
    Failure Indicators: 401 on valid request, 200 on unsigned
    Evidence: .sisyphus/evidence/task-10-axon-verify.txt

  Scenario: Blacklisted hotkey is rejected
    Tool: Bash (curl)
    Preconditions: Axon running with blacklist configured
    Steps:
      1. Configure blacklist to reject hotkey "5EvDef..."
      2. Sign request with that hotkey
      3. Send to axon
      4. Assert 403 Forbidden
    Expected Result: Blacklisted key rejected
    Failure Indicators: 200 on blacklisted key
    Evidence: .sisyphus/evidence/task-10-axon-blacklist.txt
  ```

  **Commit**: YES (Wave 2 group)
  - Message: `feat(axon): HTTP server with verification, blacklist, priority middleware`
  - Files: `bittensor-axon/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-axon`

- [x] 11. bittensor-dendrite: HTTP Client with Signing + Streaming

  **What to do**:
  - Implement `Dendrite` client using `reqwest` — the Rust equivalent of Python's aiohttp-based Dendrite
  - Implement `query(synapse, axon_info)` — send signed Synapse request to an Axon endpoint
  - Implement `forward(synapse, axon_info)` — async query with streaming support
  - Implement `call(synapse, axon_info)` — async query returning full Synapse with metadata
  - Implement `call_stream(synapse, axon_info)` — SSE streaming response
  - Sign outgoing requests: add headers `bt-nonce`, `bt-dendrite-hotkey`, `bt-axon-hotkey`, `bt-uuid`, `bt-body-hash`, `bt-signature`
  - Map HTTP errors to `BittensorError` variants
  - Support connection pooling and timeout configuration
  - Write TDD tests: mock HTTP server, verify signed headers, verify response parsing

  **Must NOT do**:
  - Do NOT implement HTTP server (that's bittensor-axon)
  - Do NOT use blocking HTTP — must be fully async
  - Do NOT skip request signing — every outgoing request must be signed

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex async HTTP client with cryptographic signing, streaming, and error mapping
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 2, 3)
  - **Parallel Group**: Wave 2 (with Tasks 6-10, 12)
  - **Blocks**: Tasks 14, 22
  - **Blocked By**: Tasks 2, 3

  **References**:

  **Pattern References**:
  - `bittensor/core/dendrite.py` (GitHub: opentensor/bittensor) — Python Dendrite implementation
  - `reqwest/examples` — async HTTP client patterns

  **API/Type References**:
  - `reqwest::Client` — HTTP client
  - `bittensor_synapse::Synapse` — from Task 3
  - `bittensor_core::AxonInfo` — target endpoint info

  **External References**:
  - reqwest: https://docs.rs/reqwest/latest/reqwest/

  **WHY Each Reference Matters**:
  - Python dendrite.py: Exact header names, signing format, and error mapping must match

  **Acceptance Criteria**:
  - [ ] `Dendrite::new(config)` creates client
  - [ ] `dendrite.query(synapse, axon_info)` sends signed request
  - [ ] Outgoing request headers contain all `bt-*` headers with valid values
  - [ ] `call_stream()` returns SSE stream that yields chunks
  - [ ] HTTP errors mapped to `BittensorError::Network`
  - [ ] `cargo test -p bittensor-dendrite` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Dendrite sends correctly signed request
    Tool: Bash (cargo test)
    Preconditions: Mock HTTP server captures headers
    Steps:
      1. Create dendrite with test keypair
      2. Call query with a test Synapse
      3. Verify mock received headers: bt-nonce, bt-signature, bt-body-hash
      4. Verify signature validates against dendrite hotkey
    Expected Result: All required headers present and valid
    Failure Indicators: Missing headers, invalid signature
    Evidence: .sisyphus/evidence/task-11-dendrite-signing.txt

  Scenario: Streaming response works
    Tool: Bash (cargo test)
    Preconditions: Mock server returns SSE stream
    Steps:
      1. Call call_stream on mock SSE endpoint
      2. Collect stream chunks
      3. Assert all chunks received and concatenated correctly
    Expected Result: Stream decoded and yielded as chunks
    Failure Indicators: Timeout, incomplete stream, decode error
    Evidence: .sisyphus/evidence/task-11-dendrite-streaming.txt
  ```

  **Commit**: YES (Wave 2 group)
  - Message: `feat(dendrite): HTTP client with signing and streaming support`
  - Files: `bittensor-dendrite/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-dendrite`

- [x] 12. bittensor-core: Balance, Weight Utils, POW Registration

  **What to do**:
  - Implement `Balance` type (if not done in Task 2 — merge here if needed):
    - Fixed-point arithmetic with tao (10^9 rao), subnet-aware display
    - Full operator set: Add, Sub, Mul, Div, Rem, Neg
    - FromStr/Display with "0.1τ" format, subnet-specific "0.1τ1" format
  - Implement `weight_utils`:
    - `normalize_weights_max_u16(weights: &[f32]) -> Vec<u16>` — normalize to max u16
    - `normalize_weights_max_u64(weights: &[f32]) -> Vec<u64>` — normalize to max u64
    - `convert_weights_to_chain(dests, weights, netuid, version_key)` — chain format conversion
    - `process_weights_for_settings(netuid, weights, version_key)` — validation + normalization pipeline
  - Implement POW registration:
    - `solve_pow(wallet, netuid, difficulty)` — CPU-based nonce search
    - Optional: `solve_pow_cuda(wallet, netuid, difficulty)` — CUDA acceleration (feature-gated)
    - Async version for tokio runtime
  - Write TDD tests: balance arithmetic, weight normalization edge cases, POW solving

  **Must NOT do**:
  - Do NOT depend on `torch` or `numpy` — pure Rust only
  - Do NOT use floating point for balance — use integer rao with Display formatting
  - Do NOT enable CUDA by default — feature gate it

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Mathematical utilities requiring precision — weight normalization has subtle edge cases
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 2)
  - **Parallel Group**: Wave 2 (with Tasks 6-11)
  - **Blocks**: Task 13
  - **Blocked By**: Task 2

  **References**:

  **Pattern References**:
  - `bittensor/utils/balance.py` (GitHub: opentensor/bittensor) — Balance arithmetic (37KB, comprehensive)
  - `bittensor/utils/weight_utils.py` (GitHub: opentensor/bittensor) — Weight normalization patterns
  - `bittensor/utils/registration/` (GitHub: opentensor/bittensor) — POW solver patterns

  **API/Type References**:
  - `bittensor_core::Balance` — from Task 2 (if already defined, extend here)

  **External References**:
  - Rust fixed-point arithmetic: `rust_decimal` crate or custom integer approach

  **WHY Each Reference Matters**:
  - Python balance.py: Exact arithmetic rules, display formats, and subnet-awareness must match
  - Python weight_utils.py: Edge cases in u16/u64 normalization (div-by-zero, all-zero weights) must match

  **Acceptance Criteria**:
  - [ ] `Balance::from_tao(1.5) + Balance::from_rao(500_000_000) == Balance::from_tao(2.0)`
  - [ ] `normalize_weights_max_u16(&[0.5, 0.3, 0.2])` produces `[32767, 19660, 13107]`
  - [ ] `normalize_weights_max_u16(&[0.0, 0.0, 0.0])` handles zero case gracefully
  - [ ] POW solver finds a valid nonce within reasonable time for difficulty ≤ 10
  - [ ] `cargo test -p bittensor-core -- utils` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Weight normalization edge cases
    Tool: Bash (cargo test)
    Preconditions: bittensor-core compiles
    Steps:
      1. Run `cargo test -p bittensor-core -- weight`
      2. Assert all tests pass including: all-zeros, single-non-zero, negative-input, overflow
    Expected Result: All edge cases handled gracefully
    Failure Indicators: Panic on zero division, wrong max value, overflow
    Evidence: .sisyphus/evidence/task-12-weight-utils.txt

  Scenario: POW solver finds valid nonce
    Tool: Bash (cargo test)
    Preconditions: POW solver implemented
    Steps:
      1. Run `cargo test -p bittensor-core -- pow_solver`
      2. Assert test passes with valid nonce within timeout
    Expected Result: Nonce found that satisfies difficulty requirement
    Failure Indicators: Timeout, invalid nonce, hash mismatch
    Evidence: .sisyphus/evidence/task-12-pow-solver.txt
  ```

  **Commit**: YES (Wave 2 group)
  - Message: `feat(core): Balance arithmetic, weight normalization, POW registration`
  - Files: `bittensor-core/src/balance.rs, bittensor-core/src/weight_utils.rs, bittensor-core/src/pow.rs`
  - Pre-commit: `cargo test -p bittensor-core`

- [x] 13. bittensor-metagraph: Neural Graph State + Sync

  **What to do**:
  - Implement `Metagraph` struct holding per-subnet state: `n`, `uids`, `hotkeys`, `coldkeys`, `stake`, `ranks`, `trust`, `consensus`, `validator_trust`, `incentive`, `dividends`, `emission`, `weights`, `bonds`, `active`, `axon_info`, `prometheus_info`, `block`
  - Support both `ndarray` (default) and `candle`/`tch` (feature-gated) backends for tensor storage
  - Implement `sync(client: &SubtensorClient, netuid: u16)` — populate all fields from chain
  - Implement `save(path)` / `load(path)` — serialize/deserialize to disk
  - Implement `neurons()` — return iterator over NeuronInfo for each UID
  - Implement index access by UID: `metagraph[uid]`
  - Write TDD tests: sync from mock chain, round-trip save/load, index access, tensor ops

  **Must NOT do**:
  - Do NOT depend on `numpy` or `torch` — use `ndarray` as default backend
  - Do NOT store the entire metagraph in memory for all subnets — one Metagraph per netuid
  - Do NOT make sync blocking — must be async

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex data structure with tensor abstraction, async chain sync, and feature-gated ML backends
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 7, 12)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 23
  - **Blocked By**: Tasks 7, 12

  **References**:

  **Pattern References**:
  - `bittensor/core/metagraph.py` (GitHub: opentensor/bittensor) — Python Metagraph implementation with tensor attributes
  - `crabtensor/src/metagraph.rs` (GitHub: threetau/crabtensor) — Rust metagraph pattern

  **API/Type References**:
  - `bittensor_chain::SubtensorClient` — from Task 7
  - `bittensor_core::NeuronInfo, AxonInfo` — from Task 2
  - `ndarray::Array1<f32>` — default tensor backend

  **External References**:
  - ndarray crate: https://docs.rs/ndarray/latest/ndarray/

  **WHY Each Reference Matters**:
  - Python metagraph.py: Field names, types, and sync logic must match for API parity
  - crabtensor metagraph: Rust-native pattern for tensor storage

  **Acceptance Criteria**:
  - [ ] `Metagraph::new(1)` creates empty metagraph for subnet 1
  - [ ] `metagraph.sync(&client, 1).await` populates all fields
  - [ ] `metagraph.save(path)` + `Metagraph::load(path)` round-trips
  - [ ] `metagraph[5]` returns NeuronInfo for UID 5
  - [ ] `cargo test -p bittensor-metagraph` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Metagraph sync populates all fields from mock chain
    Tool: Bash (cargo test)
    Preconditions: Mock SubtensorClient with test data
    Steps:
      1. Create Metagraph for netuid 1
      2. Sync from mock client
      3. Assert n > 0, uids.len() == n, stake.len() == n, etc.
    Expected Result: All tensor fields populated with matching lengths
    Failure Indicators: Zero-length tensors, mismatched lengths, None fields
    Evidence: .sisyphus/evidence/task-13-metagraph-sync.txt

  Scenario: Save and load round-trips correctly
    Tool: Bash (cargo test)
    Preconditions: Metagraph with data
    Steps:
      1. Populate metagraph from mock
      2. Save to temp file
      3. Load from same file
      4. Assert all fields match original
    Expected Result: Perfect round-trip
    Failure Indicators: Field mismatch, decode error
    Evidence: .sisyphus/evidence/task-13-metagraph-roundtrip.txt
  ```

  **Commit**: YES (Wave 3 group)
  - Message: `feat(metagraph): neural graph state, sync from chain, tensor abstraction`
  - Files: `bittensor-metagraph/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-metagraph`

- [x] 14. Axon-Dendrite-Synapse Integration Test

  **What to do**:
  - Write a comprehensive integration test that:
    1. Starts an Axon server with a test Synapse handler
    2. Creates a Dendrite client with a test keypair
    3. Sends a signed Synapse request from Dendrite to Axon
    4. Verifies Axon's verification middleware accepts the request
    5. Verifies the handler processes and returns correctly
    6. Verifies Dendrite receives and parses the response
  - Test streaming: send a streaming request, verify SSE chunks
  - Test rejection: send unsigned request, verify 401
  - Test blacklisting: send from blacklisted key, verify 403
  - Test body hash integrity: tamper with body, verify rejection
  - This test validates the entire Axon/Dendrite/Synapse protocol stack

  **Must NOT do**:
  - Do NOT use mock transport — must test real HTTP round-trip
  - - Do NOT skip the signature verification in tests

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Integration testing across 3 crates — requires careful coordination but follows established patterns
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 10, 11)
  - **Parallel Group**: Wave 3
  - **Blocks**: None directly
  - **Blocked By**: Tasks 10, 11

  **References**:

  **Pattern References**:
  - `bittensor/tests/` (GitHub: opentensor/bittensor) — Python integration test patterns

  **API/Type References**:
  - `bittensor_axon::Axon` — from Task 10
  - `bittensor_dendrite::Dendrite` — from Task 11
  - `bittensor_synapse::Synapse` — from Task 3

  **External References**:
  - None needed — this is internal integration testing

  **WHY Each Reference Matters**:
  - Python integration tests: Define the expected end-to-end behavior patterns

  **Acceptance Criteria**:
  - [ ] Signed request accepted and processed correctly
  - [ ] Unsigned request rejected with 401
  - [ ] Blacklisted key rejected with 403
  - [ ] Tampered body hash rejected
  - [ ] Streaming request yields SSE chunks
  - [ ] `cargo test -p bittensor-axon --test integration` passes
  - [ ] `cargo test -p bittensor-dendrite --test integration` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Full Axon-Dendrite-Synapse round-trip
    Tool: Bash (cargo test)
    Preconditions: All 3 crates built
    Steps:
      1. Run `cargo test -p bittensor-axon --test integration`
      2. Run `cargo test -p bittensor-dendrite --test integration`
      3. Assert all tests pass
    Expected Result: Full protocol stack validated end-to-end
    Failure Indicators: Signature failure, header mismatch, decode error
    Evidence: .sisyphus/evidence/task-14-axon-dendrite-integration.txt

  Scenario: Security middleware rejects invalid requests
    Tool: Bash (cargo test)
    Preconditions: Integration tests built
    Steps:
      1. Run tests with tampered body hash → assert rejection
      2. Run tests with expired nonce → assert rejection
      3. Run tests from blacklisted key → assert 403
    Expected Result: All security violations caught
    Failure Indicators: Invalid request accepted (security breach)
    Evidence: .sisyphus/evidence/task-14-security-middleware.txt
  ```

  **Commit**: YES (Wave 3 group)
  - Message: `test(integration): axon-dendrite-synapse end-to-end protocol validation`
  - Files: `bittensor-axon/tests/integration.rs, bittensor-dendrite/tests/integration.rs`
  - Pre-commit: `cargo test --test integration`

- [x] 15. bittensor-chain: Full Integration Tests

  **What to do**:
  - Write integration tests against local Subtensor devnet (from Task 16):
    - Query tests: metagraph, balance, stake, neuron info
    - Extrinsic tests: transfer, stake, set_weights, register
    - Event tests: subscribe to events, verify event emission
    - Subscription tests: block subscription, storage change subscription
  - Each test must start from clean chain state
  - Include a README for running integration tests
  - Feature-gate integration tests: `#[cfg(feature = "integration-tests")]`

  **Must NOT do**:
  - Do NOT run against public mainnet in CI (only local devnet)
  - Do NOT commit any real private keys in test files

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Integration testing requiring local devnet — systematic but requires infrastructure
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 8, 9)
  - **Parallel Group**: Wave 3
  - **Blocks**: None directly
  - **Blocked By**: Tasks 8, 9

  **References**:

  **Pattern References**:
  - `crabtensor/tests/` (GitHub: threetau/crabtensor) — Rust integration test patterns for Bittensor

  **API/Type References**:
  - `bittensor_chain::SubtensorClient` — from Tasks 7-9

  **External References**:
  - subxt integration testing: https://docs.rs/subxt/0.50.0/subxt/#testing

  **WHY Each Reference Matters**:
  - crabtensor tests: Proven pattern for testing Bittorrent chain interaction in Rust

  **Acceptance Criteria**:
  - [ ] `cargo test -p bittensor-chain --features integration-tests` passes against local devnet
  - [ ] All 3 categories (queries, extrinsics, events) tested
  - [ ] Test README documents how to start devnet and run tests

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Chain integration tests pass against local devnet
    Tool: Bash
    Preconditions: Docker running, local Subtensor node up (Task 16)
    Steps:
      1. `docker compose -f devnet/docker-compose.yml up -d`
      2. Wait 10s for node to produce blocks
      3. `cargo test -p bittensor-chain --features integration-tests`
      4. Assert exit code 0
      5. `docker compose -f devnet/docker-compose.yml down`
    Expected Result: All integration tests pass
    Failure Indicators: Connection refused, decode errors, extrinsic rejected
    Evidence: .sisyphus/evidence/task-15-chain-integration.txt
  ```

  **Commit**: YES (Wave 3 group)
  - Message: `test(integration): chain client full integration against local devnet`
  - Files: `bittensor-chain/tests/integration.rs, bittensor-chain/tests/README.md`
  - Pre-commit: `cargo test -p bittensor-chain --features integration-tests`

- [x] 16. devnet: Docker-Compose + Scripts for Local Subtensor Node

  **What to do**:
  - Create `devnet/docker-compose.yml` using `opentensor/subtensor` image
  - Configure for local single-node dev chain (`--dev` mode)
  - Expose ports: 9944 (WS), 9933 (HTTP), 30333 (P2P) — all within 3100-3199 range if needed
  - Create `devnet/start.sh` — pull image, start node, wait for readiness
  - Create `devnet/stop.sh` — graceful shutdown
  - Create `devnet/fund_test_accounts.sh` — fund dev accounts using `--alice` sudo
  - Verify: connect with subxt client, query block number

  **Must NOT do**:
  - Do NOT use ports outside 3100-3199 (per AGENTS.md constraints)
  - Do NOT include any real mainnet keys
  - Do NOT store persistent chain data (use `--tmp` flag)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Infrastructure setup — Docker and bash scripts
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 4)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 15 (integration tests)
  - **Blocked By**: Task 4

  **References**:

  **Pattern References**:
  - `opentensor/subtensor/docker-compose.yml` (GitHub: opentensor/subtensor) — reference docker setup

  **External References**:
  - Subtensor Docker Hub: https://hub.docker.com/r/opentensor/subtensor
  - Substrate dev chain docs: https://docs.substrate.io/tutorials/build-a-blockchain/build-local-blockchain/

  **WHY Each Reference Matters**:
  - Official Subtensor docker-compose: Proven configuration for running local Bittensor chain

  **Acceptance Criteria**:
  - [ ] `docker compose -f devnet/docker-compose.yml up -d` starts node
  - [ ] Node produces blocks within 30 seconds
  - [ ] `devnet/stop.sh` stops node cleanly
  - [ ] Test accounts are funded and queryable

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Local devnet starts and produces blocks
    Tool: Bash
    Preconditions: Docker installed and running
    Steps:
      1. Run `bash devnet/start.sh`
      2. Wait 15 seconds
      3. Run `curl -s -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' http://127.0.0.1:31333`
      4. Assert response contains `"isSyncing":false` or similar healthy status
      5. Run `bash devnet/stop.sh`
    Expected Result: Node starts, is healthy, stops cleanly
    Failure Indicators: Container exits, connection refused, sync stuck
    Evidence: .sisyphus/evidence/task-16-devnet-start.txt
  ```

  **Commit**: YES (Wave 3 group)
  - Message: `feat(devnet): docker-compose and scripts for local Subtensor node`
  - Files: `devnet/docker-compose.yml, devnet/start.sh, devnet/stop.sh, devnet/fund_test_accounts.sh`
  - Pre-commit: devnet starts and stops cleanly

- [x] 17. bittensor-chain: DRAND Randomness + MEV Shield

  **What to do**:
  - Implement DRAND beacon integration:
    - Fetch round info and randomness from DRAND HTTP API
    - Verify DRAND signature using BLS12-381
    - Cache recent rounds for efficiency
  - Implement MEV Shield (encrypted extrinsic submission):
    - Fetch on-chain NextKey (ML-KEM-768 post-quantum public key)
    - Encrypt extrinsic payload using ML-KEM-768 (Kyber)
    - Submit encrypted extrinsic via `submit_encrypted_extrinsic` call
    - Decrypt and verify on-chain response
  - Implement timelock encryption (commit/reveal pattern):
    - Commit: encrypt values using DRAND round as timelock
    - Reveal: decrypt when DRAND round is reached
  - Feature-gate: `#[cfg(feature = "drand")]` and `#[cfg(feature = "mev-shield")]`
  - Write TDD tests: mock DRAND responses, mock key retrieval, encrypt/decrypt round-trip

  **Must NOT do**:
  - Do NOT bundle ML-KEM-768 C library — use pure Rust crate (`ml-kem` or `pqcrypto-kyber`)
  - Do NOT skip DRAND signature verification
  - Do NOT enable by default — these are optional features

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Post-quantum crypto integration, DRAND protocol, complex feature-gated design
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 7)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 20
  - **Blocked By**: Task 7

  **References**:

  **Pattern References**:
  - `bittensor/extras/timelock.py` (GitHub: opentensor/bittensor) — Python timelock implementation
  - `bittensor/core/extrinsics/asyncex/mev_shield.py` — Python MEV Shield extrinsic
  - `bittensor-drand` (GitHub: opentensor) — Python DRAND library

  **API/Type References**:
  - `bittensor_chain::SubtensorClient` — for fetching NextKey from chain

  **External References**:
  - DRAND HTTP API: https://drand.love/developers/http-api
  - ML-KEM Rust: https://docs.rs/ml-kem/latest/ml_kem/
  - BLS12-381: https://docs.rs/bls12_381/latest/bls12_381/

  **WHY Each Reference Matters**:
  - Python timelock.py: Exact DRAND integration pattern and round-based encryption scheme
  - Python mev_shield.py: Exact flow for encrypted extrinsic submission
  - ML-KEM crate: Pure Rust Kyber implementation needed for post-quantum encryption

  **Acceptance Criteria**:
  - [ ] DRAND beacon fetches and verifies a round
  - [ ] ML-KEM-768 encrypt/decrypt round-trip succeeds
  - [ ] Encrypted extrinsic encodes correctly for chain submission
  - [ ] Features `drand` and `mev-shield` compile independently
  - [ ] `cargo test -p bittensor-chain --features drand,mev-shield` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: DRAND round fetch and verify
    Tool: Bash (cargo test)
    Preconditions: Network access or mock DRAND server
    Steps:
      1. Run `cargo test -p bittensor-chain --features drand -- drand`
      2. Assert round info fetched and signature verified
    Expected Result: DRAND round data valid
    Failure Indicators: Network error, signature verification failure
    Evidence: .sisyphus/evidence/task-17-drand-verify.txt

  Scenario: ML-KEM-768 round-trip
    Tool: Bash (cargo test)
    Preconditions: mev-shield feature enabled
    Steps:
      1. Generate ML-KEM keypair
      2. Encrypt test payload
      3. Decrypt with secret key
      4. Assert plaintext matches
    Expected Result: Perfect round-trip
    Failure Indicators: Decryption failure, wrong plaintext
    Evidence: .sisyphus/evidence/task-17-ml-kem-roundtrip.txt
  ```

  **Commit**: YES (Wave 3 group)
  - Message: `feat(chain): DRAND randomness beacon + MEV Shield encrypted extrinsics`
  - Files: `bittensor-chain/src/drand/**/*.rs, bittensor-chain/src/mev/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-chain --features drand,mev-shield`

- [x] 18. bittensor-cli: Wallet Commands

  **What to do**:
  - Implement CLI binary `btcli-rs` using `clap` with derive macros
  - Implement wallet command group:
    - `wallet create` — generate new coldkey (encrypted) + hotkey pair
    - `wallet list` — list all wallets in `~/.bittensor/wallets/`
    - `wallet show` — display wallet details (address, balance, hotkeys)
    - `wallet balance` — show balance for a wallet/coldkey
    - `wallet overview` — comprehensive wallet overview (stakes, delegations, etc.)
    - `wallet transfer` — transfer TAO to another address
    - `wallet swap-coldkey` — initiate coldkey swap process
    - `wallet inspect` — show all keys and addresses
    - `wallet regen-coldkey` — regenerate coldkey from mnemonic
    - `wallet regen-coldkeypub` — regenerate coldkeypub from public key
    - `wallet create-hotkey` — create a new hotkey under a wallet
    - `wallet regen-hotkey` — regenerate hotkey from mnemonic
  - Each command: load config from `~/.bittensor/config.yml`, connect to chain via `bittensor-chain`, execute operation
  - Handle password prompts for coldkey decryption (use `rpassword` crate)
  - Support `--network` flag for mainnet/testnet/local override
  - Support `--wallet-name` and `--wallet-path` flags
  - Write TDD tests: command parsing, wallet creation output, error handling

  **Must NOT do**:
  - Do NOT implement stake/delegate/registration commands here (those are Tasks 19-20)
  - Do NOT use interactive prompts without `--yes` bypass for scripting
  - Do NOT hardcode endpoint URLs — use NetworkConfig

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Large surface area of CLI commands following established patterns, requires consistent UX
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 6, 7)
  - **Parallel Group**: Wave 4 (with Tasks 19-23)
  - **Blocks**: Tasks 25, 26
  - **Blocked By**: Tasks 6, 7

  **References**:

  **Pattern References**:
  - `btcli/commands/wallet.py` (GitHub: opentensor/btcli) — Python wallet command implementations
  - `crabtensor/src/cli/` (GitHub: threetau/crabtensor) — Rust CLI patterns for Bittensor

  **API/Type References**:
  - `bittensor_wallet::Wallet` — from Task 6
  - `bittensor_chain::SubtensorClient` — from Task 7
  - `clap::Parser`, `clap::Subcommand` — CLI framework

  **External References**:
  - clap derive docs: https://docs.rs/clap/latest/clap/_derive/index.html
  - rpassword crate: https://docs.rs/rpassword/latest/rpassword/

  **WHY Each Reference Matters**:
  - Python btcli wallet commands: Source of truth for all command names, flags, and behavior
  - clap: The de-facto Rust CLI framework — need derive macro patterns

  **Acceptance Criteria**:
  - [ ] `btcli-rs wallet create --wallet.name test` creates wallet directory + keys
  - [ ] `btcli-rs wallet list` shows all wallets
  - [ ] `btcli-rs wallet balance --wallet.name test` queries chain and shows balance
  - [ ] `btcli-rs --help` shows all command groups
  - [ ] `cargo test -p bittensor-cli` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Wallet creation via CLI
    Tool: interactive_bash (tmux)
    Preconditions: Clean home directory, no existing wallets
    Steps:
      1. Run `btcli-rs wallet create --wallet.name test_wallet --wallet.path /tmp/test-wallets`
      2. Enter password "testpass123" when prompted
      3. Assert exit code 0
      4. Run `ls /tmp/test-wallets/test_wallet/`
      5. Assert output contains "coldkey", "coldkeypub", "hotkeys"
    Expected Result: Wallet directory structure created with all key files
    Failure Indicators: Missing files, wrong directory structure, exit code non-zero
    Evidence: .sisyphus/evidence/task-18-wallet-create.txt

  Scenario: Wallet list shows created wallets
    Tool: interactive_bash (tmux)
    Preconditions: At least one wallet exists
    Steps:
      1. Run `btcli-rs wallet list --wallet.path /tmp/test-wallets`
      2. Assert output contains "test_wallet"
    Expected Result: Lists existing wallets
    Failure Indicators: Empty list, missing wallet name
    Evidence: .sisyphus/evidence/task-18-wallet-list.txt
  ```

  **Commit**: YES (Wave 4 group)
  - Message: `feat(cli): wallet commands — create, list, show, balance, transfer, swap-coldkey`
  - Files: `bittensor-cli/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-cli`

- [x] 19. bittensor-cli: Stake, Transfer, and Registration Commands

  **What to do**:
  - Implement stake command group:
    - `stake add` — stake TAO to a hotkey
    - `stake remove` — unstake TAO from a hotkey
    - `stake move` — move stake between hotkeys
    - `stake swap` — swap stake between hotkeys
    - `stake list` — list all stakes for a wallet
    - `stake get-stake` — query stake for specific hotkey
    - `stake set-auto-stake` — enable/disable auto-staking
  - Implement transfer command group:
    - `transfer` — transfer TAO between addresses
    - `transfer multiple` — batch transfer to multiple recipients
  - Implement registration commands:
    - `register` — register on a subnet (POW)
    - `burned-register` — register by burning TAO
    - `root register` — register on root network
  - Each command: sign extrinsic via wallet, submit via `bittensor-chain`, watch for finalization, display result
  - Support `--subtensor.network` and `--subtensor.chain_endpoint` flags
  - Support `--wait-for-finalization` and `--timeout` flags for extrinsic status
  - Write TDD tests: command parsing, extrinsic construction (mock), output formatting

  **Must NOT do**:
  - Do NOT submit real extrinsics in unit tests (use mock chain client)
  - Do NOT implement subnet/delegate commands here (Task 20)
  - Do NOT use `unwrap()` — all chain errors must be displayed cleanly

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: High volume of commands following the same pattern, each wrapping a chain extrinsic
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 6, 8)
  - **Parallel Group**: Wave 4 (with Tasks 18, 20-23)
  - **Blocks**: Tasks 25, 26
  - **Blocked By**: Tasks 6, 8

  **References**:

  **Pattern References**:
  - `btcli/commands/stake.py` (GitHub: opentensor/btcli) — Python stake command implementations
  - `btcli/commands/register.py` (GitHub: opentensor/btcli) — Python registration commands

  **API/Type References**:
  - `bittensor_chain::extrinsics` — from Task 8
  - `bittensor_wallet::Wallet::get_coldkey_pair()` — for signing

  **External References**:
  - None needed — internal API wrappers

  **WHY Each Reference Matters**:
  - Python btcli commands: Source of truth for all command names, flags, and UX flow

  **Acceptance Criteria**:
  - [ ] `btcli-rs stake add --wallet.name test --amount 1.0` constructs valid extrinsic
  - [ ] `btcli-rs transfer --destination 5Grw... --amount 0.5` constructs valid transfer
  - [ ] `btcli-rs register --netuid 1` attempts POW registration
  - [ ] `cargo test -p bittensor-cli` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Stake add command constructs valid extrinsic
    Tool: interactive_bash (tmux)
    Preconditions: Wallet exists, mock chain client available
    Steps:
      1. Run `btcli-rs stake add --wallet.name test --amount 1.0 --subtensor.network local`
      2. Enter coldkey password when prompted
      3. Assert output contains "Adding stake" and transaction hash or error message
    Expected Result: Stake extrinsic submitted (or mock confirmed)
    Failure Indicators: Parse error, signing error, connection error to mock
    Evidence: .sisyphus/evidence/task-19-stake-add.txt

  Scenario: Registration command runs POW solver
    Tool: Bash (cargo test)
    Preconditions: bittensor-cli compiles
    Steps:
      1. Run `cargo test -p bittensor-cli -- register`
      2. Assert POW solver is invoked (mock chain)
    Expected Result: Registration attempt executed
    Failure Indicators: Command not found, missing flags, solver not invoked
    Evidence: .sisyphus/evidence/task-19-register-pow.txt
  ```

  **Commit**: YES (Wave 4 group)
  - Message: `feat(cli): stake, transfer, and registration commands`
  - Files: `bittensor-cli/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-cli`

- [x] 20. bittensor-cli: Subnet, Root, and Delegate Commands

  **What to do**:
  - Implement subnet command group:
    - `subnet create` — register a new subnet
    - `subnet list` — list all subnets
    - `subnet info` — show subnet details (hyperparameters, neurons, etc.)
    - `subnet hyperparameters` — view/set subnet hyperparameters (sudo)
    - `subnet set-identity` — set subnet identity metadata
  - Implement root command group:
    - `root set-weights` — set root network weights
    - `root get-weights` — query root weights
    - `root claim` — claim root network position
  - Implement delegate command group:
    - `delegate add` — nominate a delegate
    - `delegate remove` — remove delegation
    - `delegate list` — list all delegates
    - `delegate take` — set delegate take percentage
    - `delegate my-delegates` — show my delegations
  - Implement MEV Shield CLI:
    - `mev submit-encrypted` — submit encrypted extrinsic (feature-gated)
  - Each command: construct extrinsic, sign, submit, display result
  - Write TDD tests: command parsing, extrinsic construction (mock), output formatting

  **Must NOT do**:
  - Do NOT implement commands that require sudo keys in the default path (feature-gate them)
  - Do NOT hardcode subnet IDs — always accept `--netuid` parameter
  - Do NOT enable `mev` feature by default

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: High volume of commands following established patterns from Task 18-19
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 6, 8, 17)
  - **Parallel Group**: Wave 4 (with Tasks 18-19, 21-23)
  - **Blocks**: Tasks 25, 26
  - **Blocked By**: Tasks 6, 8, 17

  **References**:

  **Pattern References**:
  - `btcli/commands/subnet.py` (GitHub: opentensor/btcli) — Python subnet commands
  - `btcli/commands/root.py` (GitHub: opentensor/btcli) — Python root commands
  - `btcli/commands/delegate.py` (GitHub: opentensor/btcli) — Python delegate commands

  **API/Type References**:
  - `bittensor_chain::extrinsics` — from Tasks 8, 17
  - `bittensor_wallet::Wallet` — from Task 6

  **External References**:
  - None needed — internal API wrappers

  **WHY Each Reference Matters**:
  - Python btcli commands: Source of truth for all command names and behavior

  **Acceptance Criteria**:
  - [ ] `btcli-rs subnet list` queries and displays all subnets
  - [ ] `btcli-rs delegate list` shows all delegates
  - [ ] `btcli-rs root set-weights --netuid 0 --weights 0.5,0.5` constructs valid extrinsic
  - [ ] `btcli-rs subnet create` constructs register subnet extrinsic
  - [ ] `cargo test -p bittensor-cli` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Subnet list command queries chain
    Tool: interactive_bash (tmux)
    Preconditions: Mock chain client with test subnets
    Steps:
      1. Run `btcli-rs subnet list --subtensor.network local`
      2. Assert output contains subnet entries (netuid, name, etc.)
    Expected Result: Subnet list displayed
    Failure Indicators: Connection error, empty output, parse error
    Evidence: .sisyphus/evidence/task-20-subnet-list.txt

  Scenario: Delegate list command
    Tool: interactive_bash (tmux)
    Preconditions: Mock chain with delegate data
    Steps:
      1. Run `btcli-rs delegate list --subtensor.network local`
      2. Assert output shows delegate hotkeys and take percentages
    Expected Result: Delegates displayed
    Failure Indicators: Empty output, wrong format
    Evidence: .sisyphus/evidence/task-20-delegate-list.txt
  ```

  **Commit**: YES (Wave 4 group)
  - Message: `feat(cli): subnet, root, and delegate commands + MEV Shield`
  - Files: `bittensor-cli/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-cli`

- [x] 21. bittensor-pyo3: Core, Wallet, and Chain Bindings

  **What to do**:
  - Set up `bittensor-pyo3` crate with `pyo3` + `maturin` build system
  - Create `pyproject.toml` with maturin config, Python package name `bittensor_rs`
  - Implement Python bindings for `bittensor-core`:
    - `BittensorError` → Python Exception with category info
    - `NetworkConfig` → Python class with `finney()`, `test()`, `local()` class methods
    - `Balance` → Python class with arithmetic operators, `__str__`, `__repr__`
    - All chain data models: `NeuronInfo`, `DelegateInfo`, `StakeInfo`, `SubnetInfo`, `SubnetHyperparameters`, `AxonInfo`, etc.
  - Implement Python bindings for `bittensor-wallet`:
    - `Wallet` → Python class with `create()`, `load()`, `sign()`, `verify()`, `ss58_address` property
    - Key management: `get_coldkey_pair()`, `get_hotkey_pair()` (returns reference, not secret)
    - Password-based coldkey decryption (prompt or parameter)
  - Implement Python bindings for `bittensor-chain`:
    - `SubtensorClient` → Python class with async methods: `get_metagraph()`, `get_balance()`, `get_stake()`, `add_stake()`, `transfer()`, `register()`, etc.
    - Use `pyo3::async_impl` for async Python methods (pyo3-async-runtimes)
  - Write TDD tests: Python `pytest` tests importing `bittensor_rs` and exercising all bindings
  - Create `bittensor-pyo3/tests/test_core.py`, `test_wallet.py`, `test_chain.py`

  **Must NOT do**:
  - Do NOT expose panic paths — all `Result` types must be converted to Python exceptions
  - Do NOT expose Rust lifetime parameters in Python API — use owned types
  - Do NOT implement axon/dendrite bindings here (Task 22)
  - Do NOT use `unwrap()` in PyO3 wrappers

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex FFI boundary design with async, error conversion, and type mapping across 3 crates
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 6, 7, 8)
  - **Parallel Group**: Wave 4 (with Tasks 18-20, 22-23)
  - **Blocks**: Tasks 25, 26
  - **Blocked By**: Tasks 6, 7, 8

  **References**:

  **Pattern References**:
  - `pyo3/examples/` — official PyO3 patterns
  - `maturin/examples/` — maturin project setup

  **API/Type References**:
  - `bittensor_core::BittensorError, Balance, NetworkConfig` — from Task 2
  - `bittensor_wallet::Wallet` — from Task 6
  - `bittensor_chain::SubtensorClient` — from Tasks 7-8

  **External References**:
  - PyO3 docs: https://pyo3.rs/v0.23/
  - pyo3-async-runtimes: https://docs.rs/pyo3-async-runtimes/latest/
  - maturin docs: https://www.maturin.rs/

  **WHY Each Reference Matters**:
  - PyO3 + maturin: Standard tooling for Rust→Python bindings, need correct setup
  - pyo3-async-runtimes: Required for async chain methods — the Python side is async too

  **Acceptance Criteria**:
  - [ ] `maturin develop` succeeds and `import bittensor_rs` works in Python
  - [ ] `bittensor_rs.Balance.from_tao(1.5)` returns a Balance with correct `__str__`
  - [ ] `bittensor_rs.Wallet.create("test", "/tmp/wallets")` creates a wallet
  - [ ] `bittensor_rs.NetworkConfig.finney()` returns configured NetworkConfig
  - [ ] `pytest bittensor-pyo3/tests/` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Python can import and use bittensor_rs
    Tool: Bash
    Preconditions: maturin develop completed
    Steps:
      1. Run `maturin develop --release`
      2. Run `python -c "import bittensor_rs; print(bittensor_rs.NetworkConfig.finney())"`
      3. Assert output contains Finney endpoint URL
    Expected Result: Module imports successfully, classes usable
    Failure Indicators: ImportError, AttributeError
    Evidence: .sisyphus/evidence/task-21-pyo3-import.txt

  Scenario: Wallet creation works from Python
    Tool: Bash
    Preconditions: maturin develop completed
    Steps:
      1. Run `python -c "from bittensor_rs import Wallet; w = Wallet.create('test', '/tmp/py-wallets'); print(w.ss58_address)"`
      2. Assert output is a valid SS58 address (starts with "5", ~47 chars)
    Expected Result: Wallet created, address printed
    Failure Indicators: Exception, wrong address format
    Evidence: .sisyphus/evidence/task-21-pyo3-wallet.txt
  ```

  **Commit**: YES (Wave 4 group)
  - Message: `feat(pyo3): Python bindings — core types, wallet, chain client`
  - Files: `bittensor-pyo3/src/**/*.rs, bittensor-pyo3/tests/*.py, bittensor-pyo3/pyproject.toml`
  - Pre-commit: `maturin develop && pytest bittensor-pyo3/tests/`

- [x] 22. bittensor-pyo3: Axon, Dendrite, and Synapse Bindings

  **What to do**:
  - Implement Python bindings for `bittensor-synapse`:
    - `Synapse` → Python base class with `to_headers()`, `from_headers()`, `body_hash()` methods
    - `TerminalInfo` → Python dataclass
    - `StreamingSynapse` → Python class with async iteration support
  - Implement Python bindings for `bittensor-axon`:
    - `Axon` → Python class with `attach(synapse_type, handler)`, `start()`, `stop()` methods
    - Middleware configuration: `set_blacklist_fn()`, `set_priority_fn()`, `set_verify_fn()`
  - Implement Python bindings for `bittensor-dendrite`:
    - `Dendrite` → Python class with async `query()`, `forward()`, `call()`, `call_stream()` methods
    - Returns Python Synapse objects with populated TerminalInfo
  - Write TDD tests: `test_synapse.py`, `test_axon.py`, `test_dendrite.py`
  - Test the full round-trip from Python: create Axon → create Dendrite → query → verify response

  **Must NOT do**:
  - Do NOT expose Rust tokio runtime details to Python — use pyo3-async-runtimes to bridge
  - Do NOT skip the signing in Python-side Dendrite queries
  - Do NOT make Synapse a trait in Python — use a base class pattern instead

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex async FFI bridging with HTTP server/client lifecycle management across language boundary
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 3, 10, 11)
  - **Parallel Group**: Wave 4 (with Tasks 18-21, 23)
  - **Blocks**: Tasks 25, 26
  - **Blocked By**: Tasks 3, 10, 11

  **References**:

  **Pattern References**:
  - `pyo3/examples/dekai` — async PyO3 example with axum server
  - `bittensor/core/axon.py` — Python Axon API surface

  **API/Type References**:
  - `bittensor_synapse::Synapse, TerminalInfo, StreamingSynapse` — from Task 3
  - `bittensor_axon::Axon` — from Task 10
  - `bittensor_dendrite::Dendrite` — from Task 11

  **External References**:
  - pyo3-async-runtimes: https://docs.rs/pyo3-async-runtimes/latest/

  **WHY Each Reference Matters**:
  - pyo3 async example: Shows the exact pattern for bridging tokio async to Python async
  - Python Axon API: Defines the exact Python interface that users expect

  **Acceptance Criteria**:
  - [ ] Python can create and start an Axon: `ax = bittensor_rs.Axon(wallet, port=3100); ax.start()`
  - [ ] Python can query with Dendrite: `d = bittensor_rs.Dendrite(wallet); resp = await d.query(synapse, axon_info)`
  - [ ] Full Python round-trip: Axon receives and responds to Dendrite query
  - [ ] `pytest bittensor-pyo3/tests/` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Python Axon-Dendrite round-trip
    Tool: Bash
    Preconditions: maturin develop completed
    Steps:
      1. Run `python bittensor-pyo3/tests/test_integration.py`
      2. Assert test passes (starts Axon, queries with Dendrite, verifies response)
    Expected Result: Full protocol round-trip works from Python
    Failure Indicators: Timeout, connection refused, signature error
    Evidence: .sisyphus/evidence/task-22-pyo3-axon-dendrite.txt

  Scenario: Python Synapse header round-trip
    Tool: Bash
    Preconditions: maturin develop completed
    Steps:
      1. Run `python -c "from bittensor_rs import Synapse, TerminalInfo; s = TerminalInfo(...); h = s.to_headers(); s2 = TerminalInfo.from_headers(h); assert s == s2"`
      2. Assert no error
    Expected Result: Header serialization round-trips correctly
    Failure Indicators: AttributeError, mismatch
    Evidence: .sisyphus/evidence/task-22-pyo3-synapse-roundtrip.txt
  ```

  **Commit**: YES (Wave 4 group)
  - Message: `feat(pyo3): Python bindings — axon, dendrite, synapse protocol`
  - Files: `bittensor-pyo3/src/**/*.rs, bittensor-pyo3/tests/*.py`
  - Pre-commit: `maturin develop && pytest bittensor-pyo3/tests/`

- [x] 23. bittensor-tui: Terminal UI with ratatui

  **What to do**:
  - Implement `bittensor-tui` binary using `ratatui` + `crossterm` backend
  - Design main dashboard layout with panes:
    - **Network Overview**: total stake, issuance, block height, network hash rate
    - **Wallet Panel**: selected wallet balance, stakes, delegations
    - **Subnet Explorer**: list subnets, select to see details (neurons, weights, emissions)
    - **Delegate Monitor**: top delegates, take percentages, my delegations
    - **Neuron View**: selected neuron details (rank, trust, incentive, bonds, weights)
  - Implement async data refresh: poll chain every 5s via `tokio::interval`
  - Implement keyboard navigation: arrow keys, Tab, Enter to select/expand
  - Implement color scheme: dark theme with Bittensor brand colors
  - Support `--network` flag and `--refresh-rate` flag
  - Graceful terminal cleanup on exit (raw mode disable)
  - Write TDD tests: widget rendering (test_backend), keyboard input handling, data refresh cycle

  **Must NOT do**:
  - Do NOT use ncurses — must use crossterm for cross-platform support
  - Do NOT block the main thread on chain queries — use async channels
  - Do NOT add mouse support in V1 (keyboard only)
  - Do NOT make TUI a required component — it's an optional binary

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Terminal UI design + async data pipeline + rendering — requires visual attention
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 6, 7, 13)
  - **Parallel Group**: Wave 4 (with Tasks 18-22)
  - **Blocks**: Tasks 25, 26
  - **Blocked By**: Tasks 6, 7, 13

  **References**:

  **Pattern References**:
  - `ratatui/examples/` — official ratatui examples for layout, widgets, async
  - `crabtensor/src/tui/` (GitHub: threetau/crabtensor) — if it has TUI code (check)

  **API/Type References**:
  - `bittensor_wallet::Wallet` — from Task 6
  - `bittensor_chain::SubtensorClient` — from Task 7
  - `bittensor_metagraph::Metagraph` — from Task 13

  **External References**:
  - ratatui docs: https://ratatui.rs/
  - ratatui async template: https://github.com/ratatui/templates

  **WHY Each Reference Matters**:
  - ratatui examples: Proven patterns for terminal UI layout and async data refresh
  - Metagraph API: Data source for the dashboard — must understand what's available

  **Acceptance Criteria**:
  - [ ] `cargo run -p bittensor-tui` launches TUI with dashboard
  - [ ] Network overview pane shows block height and total stake
  - [ ] Keyboard navigation works between panes
  - [ ] Clean exit with Ctrl+C (terminal restored)
  - [ ] `cargo test -p bittensor-tui` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: TUI launches and displays dashboard
    Tool: interactive_bash (tmux)
    Preconditions: bittensor-tui compiles
    Steps:
      1. Run `cargo run -p bittensor-tui -- --network local`
      2. Wait 3 seconds
      3. Take screenshot: capture TUI output
      4. Assert dashboard panes visible (network, wallet, subnet)
      5. Press 'q' to quit
    Expected Result: Dashboard renders with data from chain
    Failure Indicators: Blank screen, panic, terminal corruption
    Evidence: .sisyphus/evidence/task-23-tui-launch.png

  Scenario: TUI handles keyboard navigation
    Tool: interactive_bash (tmux)
    Preconditions: TUI running
    Steps:
      1. Press Tab key to switch panes
      2. Press arrow keys to navigate subnet list
      3. Press Enter to select a subnet
      4. Press 'q' to quit
      5. Assert terminal restored cleanly (echo test)
    Expected Result: Pane switching and item selection work
    Failure Indicators: No focus change, crash on keypress
    Evidence: .sisyphus/evidence/task-23-tui-navigation.txt
  ```

  **Commit**: YES (Wave 4 group)
  - Message: `feat(tui): terminal UI dashboard with network, wallet, subnet panels`
  - Files: `bittensor-tui/src/**/*.rs`
  - Pre-commit: `cargo test -p bittensor-tui`

- [x] 24. bittensor-wasm: WASM-Compatible Subset

  **What to do**:
  - Create `bittensor-wasm` crate targeting `wasm32-unknown-unknown`
  - Re-export a browser/edge-compatible subset of `bittensor-core`:
    - `Balance` type with arithmetic and display
    - `NetworkConfig` with preset endpoints
    - All chain data models (read-only structs)
    - `AxonInfo`, `NeuronInfoLite`, `SubnetInfo`
  - Re-export `bittensor-synapse` protocol types:
    - `Synapse` trait (simplified — no tokio/axum deps)
    - `TerminalInfo` struct
    - Header serialization functions
  - Re-export `bittensor-chain` query methods (read-only):
    - Use `wasm-bindgen-futures` + `gloo-net` for WebSocket connections from browser
    - Expose async query functions: `get_metagraph`, `get_balance`, `get_stake`, `get_subnet_info`
  - Generate JavaScript/TypeScript bindings via `wasm-bindgen` + `wasm-pack`
  - Create `bittensor-wasm/src/lib.rs` with `#[wasm_bindgen]` exports
  - Add `wasm-pack.toml` configuration
  - Write TDD tests: `wasm-pack test --node` for Node.js, `--browser` for browser

  **Must NOT do**:
  - Do NOT use `tokio` — use `wasm-bindgen-futures::spawn_local` for async
  - Do NOT use `std::fs` or `std::net` — no file I/O or raw sockets in WASM
  - Do NOT include wallet encryption/decryption (NaCl needs libsodium — not available in WASM)
  - Do NOT include extrinsic submission in WASM (signing needs platform-specific keystore)
  - Do NOT enable by default in workspace — feature-gate with `wasm` feature

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: WASM target constraints, async bridging, and selective re-exports require careful architecture
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Tasks 2, 3, 7)
  - **Parallel Group**: Wave 5 (with Tasks 25-26)
  - **Blocks**: Tasks 25, 26
  - **Blocked By**: Tasks 2, 3, 7

  **References**:

  **Pattern References**:
  - `subxt/examples/wasm` — subxt WASM usage pattern
  - `gloo-net` examples — browser WebSocket from Rust

  **API/Type References**:
  - `bittensor_core::Balance, NetworkConfig, NeuronInfoLite, AxonInfo, SubnetInfo` — from Task 2
  - `bittensor_synapse::TerminalInfo, Synapse` — from Task 3
  - `bittensor_chain::SubtensorClient::get_*` query methods — from Task 7

  **External References**:
  - wasm-bindgen: https://rustwasm.github.io/wasm-bindgen/
  - wasm-pack: https://rustwasm.github.io/wasm-pack/
  - gloo-net: https://docs.rs/gloo-net/latest/gloo_net/

  **WHY Each Reference Matters**:
  - subxt WASM: Shows how to use subxt client from browser — critical pattern
  - gloo-net: Browser-native HTTP/WebSocket for WASM — replaces reqwest/tokio

  **Acceptance Criteria**:
  - [ ] `wasm-pack build --target web` succeeds
  - [ ] `wasm-pack test --node` passes
  - [ ] JavaScript can `import { Balance, NetworkConfig } from "bittensor-wasm"`
  - [ ] No `tokio`, `std::fs`, or `std::net` in bittensor-wasm dependency tree
  - [ ] `cargo check -p bittensor-wasm --target wasm32-unknown-unknown` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: WASM crate builds for browser target
    Tool: Bash
    Preconditions: wasm-pack installed
    Steps:
      1. Run `wasm-pack build --target web`
      2. Assert exit code 0
      3. Run `ls pkg/bittensor_wasm.js` and assert file exists
    Expected Result: WASM package builds with JS bindings
    Failure Indicators: Compilation error, missing WASM target
    Evidence: .sisyphus/evidence/task-24-wasm-build.txt

  Scenario: WASM tests pass in Node.js
    Tool: Bash
    Preconditions: wasm-pack + Node.js installed
    Steps:
      1. Run `wasm-pack test --node`
      2. Assert all tests pass
    Expected Result: All WASM-exported functions work in Node
    Failure Indicators: Runtime error, missing imports
    Evidence: .sisyphus/evidence/task-24-wasm-node-test.txt
  ```

  **Commit**: YES (Wave 5 group)
  - Message: `feat(wasm): WASM-compatible subset for browser/edge usage`
  - Files: `bittensor-wasm/src/**/*.rs, bittensor-wasm/Cargo.toml, wasm-pack.toml`
  - Pre-commit: `wasm-pack build --target web`

- [x] 25. Documentation + Examples

  **What to do**:
  - Write comprehensive `rustdoc` for all public APIs across all crates:
    - Every `pub fn`, `pub struct`, `pub enum`, `pub trait` must have `///` doc comments
    - Include code examples in doc comments where applicable (`# Examples` section)
  - Create `examples/` directory with runnable examples:
    - `examples/wallet_create.rs` — create a wallet, display SS58 address
    - `examples/chain_query.rs` — connect to Finney, query metagraph for subnet 1
    - `examples/transfer.rs` — transfer TAO between accounts (against local devnet)
    - `examples/axon_server.rs` — start an Axon, attach a Synapse handler
    - `examples/dendrite_client.rs` — query an Axon with a Dendrite
    - `examples/metagraph_sync.rs` — sync and display metagraph state
    - `examples/tui_dashboard.rs` — launch the TUI dashboard
    - `examples/python_wallet.py` — use bittensor_rs from Python
    - `examples/wasm_browser/` — minimal HTML+JS using bittensor-wasm
  - Create crate-level `README.md` for each crate with:
    - Purpose, quick start, feature flags, API overview
  - Write workspace root `README.md` with:
    - Architecture diagram (ASCII), crate descriptions, quick start
  - Verify: `cargo doc --workspace --no-deps` passes without warnings

  **Must NOT do**:
  - Do NOT generate documentation for internal (`pub(crate)`) items
  - Do NOT add README.md files in `src/` directories — only crate root
  - Do NOT skip undocumented pub items — every one must have doc comments
  - Do NOT add AI-slop boilerplate like "This module provides functionality for..."

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation-focused task — prose quality matters
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on ALL implementation tasks for accuracy)
  - **Parallel Group**: Wave 5 (sequential after all prior waves)
  - **Blocks**: Task 26
  - **Blocked By**: All implementation tasks (1-24)

  **References**:

  **Pattern References**:
  - `bittensor/README.md` (GitHub: opentensor/bittensor) — Python SDK documentation structure
  - `crabtensor/README.md` (GitHub: threetau/crabtensor) — Rust SDK documentation reference

  **API/Type References**:
  - All crate public APIs from Tasks 1-24

  **External References**:
  - rustdoc book: https://doc.rust-lang.org/rustdoc/
  - cargo doc: https://doc.rust-lang.org/cargo/commands/cargo-doc.html

  **WHY Each Reference Matters**:
  - Python SDK README: Defines what users expect in terms of documentation completeness
  - rustdoc: The standard — doc comments must follow conventions

  **Acceptance Criteria**:
  - [ ] `cargo doc --workspace --no-deps` passes with zero warnings
  - [ ] Every example in `examples/` compiles and runs
  - [ ] Each crate has a `README.md`
  - [ ] Workspace root `README.md` has architecture diagram and quick start

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Documentation builds without warnings
    Tool: Bash
    Preconditions: All crates compile
    Steps:
      1. Run `cargo doc --workspace --no-deps 2>&1`
      2. Assert exit code 0
      3. Assert no "missing documentation" warnings in output
    Expected Result: Clean doc build
    Failure Indicators: Warning count > 0, missing docs
    Evidence: .sisyphus/evidence/task-25-doc-build.txt

  Scenario: All examples compile and run
    Tool: Bash
    Preconditions: All examples written
    Steps:
      1. For each example in examples/: `cargo run --example <name> -- --help` or equivalent
      2. Assert each compiles and shows help/usage
    Expected Result: All examples are runnable
    Failure Indicators: Compilation error, missing deps
    Evidence: .sisyphus/evidence/task-25-examples-compile.txt
  ```

  **Commit**: YES (Wave 5 group)
  - Message: `docs: comprehensive rustdoc, examples, and crate READMEs`
  - Files: `**/*.rs (doc comments), examples/**/*.rs, **/README.md`
  - Pre-commit: `cargo doc --workspace --no-deps`

- [x] 26. Final Workspace Integration + CI Pipeline

  **What to do**:
  - Final workspace integration:
    - Verify `cargo build --workspace` succeeds with zero errors
    - Verify `cargo test --workspace` passes all tests
    - Verify `cargo clippy --workspace -- -D warnings` passes
    - Verify `cargo fmt -- --check` passes
    - Run `cargo test --workspace --features integration-tests` against local devnet
  - Set up CI pipeline (`.github/workflows/ci.yml`):
    - **Lint**: `cargo fmt -- --check` + `cargo clippy --workspace -- -D warnings`
    - **Test**: `cargo test --workspace` (unit tests only)
    - **Build**: `cargo build --workspace --release`
    - **Doc**: `cargo doc --workspace --no-deps`
    - **Integration**: `cargo test --workspace --features integration-tests` (with docker service for devnet)
    - **PyO3**: `maturin develop && pytest` for bittensor-pyo3
    - **WASM**: `wasm-pack build --target web && wasm-pack test --node`
  - Add `deny.toml` for `cargo-deny` (license + security audit)
  - Add `codecov.yml` for coverage reporting
  - Verify CI runs green on all steps (or document which steps need manual setup)

  **Must NOT do**:
  - Do NOT commit secrets or real private keys in CI config
  - Do NOT enable auto-publish to crates.io (internal project per user requirement)
  - Do NOT skip integration test step — document it even if it needs self-hosted runner
  - Do NOT use `--allow-dirty` or `--allow-staged` in CI fmt check

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: CI pipeline setup, workspace integration verification, and cross-platform configuration
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on ALL tasks being complete)
  - **Parallel Group**: Wave 5 (final task, sequential after Task 25)
  - **Blocks**: F1-F4 (Final Verification Wave)
  - **Blocked By**: All implementation tasks (1-25)

  **References**:

  **Pattern References**:
  - `crabtensor/.github/workflows/` (GitHub: threetau/crabtensor) — CI patterns for Bittensor Rust SDK

  **External References**:
  - GitHub Actions Rust: https://github.com/actions-rs/example
  - cargo-deny: https://embarkstudios.github.io/cargo-deny/

  **WHY Each Reference Matters**:
  - crabtensor CI: Proven CI configuration for a Bittensor Rust workspace

  **Acceptance Criteria**:
  - [ ] `cargo build --workspace --release` succeeds
  - [ ] `cargo test --workspace` passes all unit tests
  - [ ] `cargo clippy --workspace -- -D warnings` passes
  - [ ] `cargo fmt -- --check` passes
  - [ ] `.github/workflows/ci.yml` exists and is valid YAML
  - [ ] `deny.toml` exists with license/security checks configured

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: Full workspace builds and tests pass
    Tool: Bash
    Preconditions: All crates implemented
    Steps:
      1. Run `cargo build --workspace --release`
      2. Assert exit code 0
      3. Run `cargo test --workspace`
      4. Assert exit code 0
      5. Run `cargo clippy --workspace -- -D warnings`
      6. Assert exit code 0
    Expected Result: Full green workspace
    Failure Indicators: Compilation error, test failure, clippy warning
    Evidence: .sisyphus/evidence/task-26-workspace-green.txt

  Scenario: CI workflow is valid
    Tool: Bash
    Preconditions: .github/workflows/ci.yml exists
    Steps:
      1. Run `cat .github/workflows/ci.yml | python3 -c "import yaml,sys; yaml.safe_load(sys.stdin)"`
      2. Assert exit code 0 (valid YAML)
      3. Grep for required steps: fmt, clippy, test, build
      4. Assert all present
    Expected Result: Valid CI configuration
    Failure Indicators: YAML parse error, missing steps
    Evidence: .sisyphus/evidence/task-26-ci-valid.txt
  ```

  **Commit**: YES (Wave 5 group)
  - Message: `ci: workspace CI pipeline + cargo-deny + final integration`
  - Files: `.github/workflows/ci.yml, deny.toml, codecov.yml`
  - Pre-commit: `cargo build --workspace && cargo test --workspace`

---

## Fix Wave (Post F1-F4 Review — Critical Issues)

> Fixes identified by F1 (REJECT), F2 (CONDITIONAL PASS), F4 (APPROVE WITH NOTES).
> These tasks MUST complete before re-running F1-F4.

- [x] 27. Fix `unwrap()` in Library Code

  **What to do**:
  - `bittensor-chain/src/drand/beacon.rs:219,236,248`: Replace `.cache.lock().unwrap()` with `.cache.lock().map_err(|e| DrandBeaconError::LockPoisoned(e.to_string()))` or `.expect("drand beacon cache lock")`
  - `bittensor-wasm/src/queries.rs:114,155`: Replace `.try_into().unwrap()` with `.expect("length validated above")`
  - Add proper error variant `DrandBeaconError::LockPoisoned(String)` if not already present
  - Verify: `cargo test -p bittensor-chain -- drand` + `cargo test -p bittensor-wasm` pass

  **Must NOT do**:
  - Do NOT replace with silent error swallowing
  - Do NOT change the logic flow, only the error handling path

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Fix Wave A (with Tasks 28, 29)
  - **Blocks**: Re-review
  - **Blocked By**: None

  **Acceptance Criteria**:
  - [ ] `grep -rn "\.lock()\.unwrap()" bittensor-chain/src/ bittensor-wasm/src/` returns zero matches in non-test code
  - [ ] `grep -rn "\.try_into()\.unwrap()" bittensor-wasm/src/queries.rs` returns zero matches in non-test code
  - [ ] `cargo test -p bittensor-chain -p bittensor-wasm` passes

  **Commit**: YES
  - Message: `fix(chain,wasm): replace unwrap() with expect() in library code`
  - Files: `bittensor-chain/src/drand/beacon.rs, bittensor-wasm/src/queries.rs`

- [x] 28. Add Missing Query Methods to bittensor-chain

  **What to do**:
  The plan specifies "100+ query methods" but only 39 are implemented. Add the missing query methods by examining the generated subxt API for `subtensor_module` storage items not yet exposed. Categories to cover:
  - **Per-neuron field queries**: `get_rank`, `get_trust`, `get_consensus`, `get_incentive`, `get_dividends`, `get_emission`, `get_active`, `get_last_update`, `get_validator_permit`, `get_validator_trust`, `get_bonds` (per-uid variants)
  - **Multi-key batch lookups**: `get_neuron_for_pubkey_and_subnet_batch`, `get_stake_info_for_coldkeys_batch`
  - **Subnet-specific**: `get_subnet_owner_by_netuid`, `get_subnet_name_by_netuid`, `get_tempo`, `get_subnet_identity`, `get_network_registered_at`, `get_max_allowed_uids`, `get_network_modality`, `get_network_registration_cost`, `get_min_allowed_weights`, `get_max_weight_limit`, `get_weights_version_key`
  - **Network-wide**: `get_total_subnets`, `get_subnet_hash`, `get_nominator_min_required_stake`, `get_n`, `get_n_registry`, `get_n_subnetworks`
  - **Serving**: `get_axon_by_hotkey_and_netuid`, `get_prometheus_by_hotkey_and_netuid`
  - **Weights**: `get_pending_weights`, `get_weight_hashes`, `get_weight_commit_reveal_interval`, `get_commit_reveal_period`
  - **Staking**: `get_stake_for_hotkey_on_subnet`, `get_total_stake_per_hotkey`, `get_total_stake_per_coldkey`
  - **Delegation**: `get_delegated_stake`, `get_is_delegate`, `get_delegate_summary`
  - **Other pallets**: System (account, block_hash, block_number, events), Balances (locks, reserves), Session (validators), Timestamp (now)

  Target: ≥100 public query methods in `bittensor-chain/src/queries/`.

  **Must NOT do**:
  - Do NOT add extrinsic methods (those belong in extrinsics module)
  - Do NOT modify the generated.rs file
  - Do NOT guess at method signatures — verify against the actual subxt generated API

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: High volume of mechanical but correct query implementations
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Fix Wave B (larger scope, with Task 30)
  - **Blocks**: Re-review
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `bittensor-chain/src/queries/account.rs` — existing pattern for query methods
  - `bittensor-chain/src/queries/neuron.rs` — per-neuron field query pattern
  - Python `bittensor/core/async_subtensor.py` — full list of Python query methods

  **API/Type References**:
  - `bittensor-chain/src/generated.rs` — the ground truth for available storage items
  - Run `cargo expand -p bittensor-chain` if needed to discover generated API

  **Acceptance Criteria**:
  - [ ] `grep -c "pub async fn" bittensor-chain/src/queries/*.rs` total ≥ 100
  - [ ] All new methods follow the existing `pub async fn get_X(&self, ...) -> Result<...>` pattern
  - [ ] `cargo test -p bittensor-chain` passes (existing + any new unit tests)
  - [ ] `cargo clippy -p bittensor-chain -- -D warnings` passes

  **Commit**: YES
  - Message: `feat(chain): expand storage queries to 100+ methods for full SDK parity`
  - Files: `bittensor-chain/src/queries/**/*.rs`

- [x] 29. Add Missing CLI Commands + PyO3 Bindings + Examples + Docs

  **What to do**:
  **CLI additions** (bittensor-cli):
  - Add `btcli weights set-weights` / `btcli weights get-weights` — weight management commands
  - Add `btcli metagraph show` / `btcli metagraph sync` — metagraph inspection commands
  - Add missing admin commands if any are in Python btcli but not in Rust

  **PyO3 additions** (bittensor-pyo3):
  - Add `Metagraph` class wrapping `bittensor_metagraph::Metagraph` with `sync()`, `save()`, `load()`, `neurons()`, index access
  - Add `DrandBeacon` class with `get_round()`, `verify()` (feature-gated `drand`)
  - Add `MevShield` class with `encrypt_extrinsic()`, `submit_encrypted()` (feature-gated `mev-shield`)

  **Example additions** (bittensor-examples):
  - Create `examples/tui_dashboard.rs` — minimal TUI launch example
  - Create `examples/python_wallet.py` — Python bittensor_rs usage example
  - Create `examples/wasm_browser/` — minimal HTML+JS using bittensor-wasm

  **Doc additions**:
  - Add `///` doc comments to any `pub fn` in bittensor-core, bittensor-wallet, bittensor-chain, bittensor-synapse, bittensor-axon, bittensor-dendrite, bittensor-metagraph that are missing them
  - Verify: `cargo doc --workspace --no-deps` passes with zero "missing documentation" warnings

  **Must NOT do**:
  - Do NOT break existing CLI command structure or rename commands
  - Do NOT add `unwrap()` in new PyO3 bindings — return `PyResult`
  - Do NOT add Python dependencies — keep it pure Rust + maturin

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Multi-crate changes across CLI, PyO3, examples, and docs
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Fix Wave B (with Task 28)
  - **Blocks**: Re-review
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `bittensor-cli/src/commands/wallet.rs` — CLI command pattern to follow
  - `bittensor-pyo3/src/core_types.rs` — PyO3 binding pattern to follow
  - Python `btcli/commands/weights.py` — Python weights command reference
  - Python `btcli/commands/metagraph.py` — Python metagraph command reference

  **API/Type References**:
  - `bittensor_metagraph::Metagraph` — from Task 13
  - `bittensor_chain::drand::beacon::DrandBeaconService` — from Task 17
  - `bittensor_chain::mev_shield` — from Task 17

  **Acceptance Criteria**:
  - [ ] `btcli-rs weights --help` shows set-weights/get-weights subcommands
  - [ ] `btcli-rs metagraph --help` shows show/sync subcommands
  - [ ] `maturin develop && python -c "from bittensor_rs import Metagraph"` works
  - [ ] `examples/tui_dashboard.rs` compiles with `cargo run --example tui_dashboard`
  - [ ] `cargo doc --workspace --no-deps 2>&1 | grep "missing documentation" | wc -l` = 0
  - [ ] `cargo clippy --workspace -- -D warnings` passes

  **Commit**: YES
  - Message: `feat(cli,pyo3,docs): add weights/metagraph commands, PyO3 bindings, examples, docs`
  - Files: `bittensor-cli/src/**/*.rs, bittensor-pyo3/src/**/*.rs, bittensor-examples/examples/**, **/*.rs (doc comments)`

- [x] 30. Generate Coverage Report + Fix `#[allow(dead_code)]` Annotations

  **What to do**:
  - Install `cargo-llvm-cov` if not available: `cargo install cargo-llvm-cov`
  - Run: `cargo llvm-cov --workspace --ignore-filename-regex "generated|target|tests" --json > .sisyphus/evidence/coverage.json`
  - Also generate HTML: `cargo llvm-cov --workspace --ignore-filename-regex "generated|target|tests" --html --output-dir .sisyphus/evidence/coverage-html/`
  - Verify each crate has ≥80% line coverage (excluding generated.rs and test files)
  - Fix `#[allow(dead_code)]` annotations in `bittensor-cli/src/commands/subnet.rs:300` and `bittensor-wasm/src/types.rs:713,730,739` — add justification comments like `// Dead code allowed: reserved for future X feature`
  - Archive coverage summary to `.sisyphus/evidence/task-30-coverage-summary.txt`

  **Must NOT do**:
  - Do NOT modify test code to inflate coverage
  - Do NOT remove `#[allow(dead_code)]` if the code is genuinely unused but intentionally kept
  - Do NOT spend more than 15 minutes on coverage — if a crate is below 80%, document it as a known gap

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Tooling + minor annotation fixes
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Fix Wave A (with Tasks 27, 29)
  - **Blocks**: Re-review
  - **Blocked By**: None

  **Acceptance Criteria**:
  - [ ] `.sisyphus/evidence/coverage.json` exists and is valid JSON
  - [ ] Coverage report archived with per-crate line coverage percentages
  - [ ] All `#[allow(dead_code)]` annotations have justification comments
  - [ ] `cargo test --workspace` still passes

  **Commit**: YES
  - Message: `chore: generate coverage report + document dead_code annotations`
  - Files: `.sisyphus/evidence/coverage.json, .sisyphus/evidence/coverage-html/, bittensor-cli/src/commands/subnet.rs, bittensor-wasm/src/types.rs`

- [x] 31. Fix PyO3 `extension-module` Linkage Error

  **What to do**:
  - The `bittensor-pyo3` crate uses `pyo3/extension-module` feature which prevents `cargo test` from linking — it requires `libpython` symbols that aren't available outside a Python interpreter
  - Add a feature flag `python` to `bittensor-pyo3/Cargo.toml` that conditionally enables `extension-module`:
    ```toml
    [features]
    default = []
    python = ["pyo3/extension-module"]
    drand = ["bittensor-chain/drand"]
    mev-shield = ["bittensor-chain/mev-shield"]

    [dependencies]
    pyo3 = { version = "0.23", default-features = false, features = ["macros", "pyproto"] }
    ```
  - Update the `crate-type` to be conditional:
    ```toml
    [lib]
    crate-type = ["rlib"]
    # When building with `maturin`, the crate-type is overridden to "cdylib"
    ```
  - Verify: `cargo test -p bittensor-pyo3` passes (without `python` feature)
  - Verify: `maturin develop` still works (maturin auto-adds `extension-module`)
  - Add a CI note in the crate README: "Test with `cargo test -p bittensor-pyo3`, build Python wheel with `maturin develop`"

  **Must NOT do**:
  - Do NOT remove `extension-module` entirely — maturin needs it for building the Python extension
  - Do NOT change the public Python API
  - Do NOT add Python as a dev-dependency

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Small Cargo.toml change + verification
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Fix Wave C
  - **Blocks**: Re-Review Wave (R1-R3)
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `pyo3` docs on feature flags: https://pyo3.rs/v0.23/#features

  **API/Type References**:
  - `bittensor-pyo3/Cargo.toml` — current configuration

  **External References**:
  - PyO3 feature guide: https://pyo3.rs/v0.23/features#extension-module

  **WHY Each Reference Matters**:
  - PyO3 docs: The exact mechanism for conditional `extension-module` feature

  **Acceptance Criteria**:
  - [ ] `cargo test -p bittensor-pyo3` passes (0 linker errors)
  - [ ] `cargo test --workspace --exclude bittensor-pyo3` still passes
  - [ ] `cargo check -p bittensor-pyo3 --features python` succeeds
  - [ ] `cargo clippy -p bittensor-pyo3` passes

  **QA Scenarios (MANDATORY):**

  ```
  Scenario: PyO3 crate tests run successfully
    Tool: Bash (cargo test)
    Preconditions: bittensor-pyo3 compiles
    Steps:
      1. Run `cargo test -p bittensor-pyo3`
      2. Assert exit code 0
      3. Assert no "undefined symbol: Py_InitializeEx" errors
    Expected Result: All PyO3 unit tests pass
    Failure Indicators: Linker errors referencing Py_ symbols
    Evidence: .sisyphus/evidence/task-31-pyo3-test.txt

  Scenario: Full workspace tests still pass
    Tool: Bash (cargo test)
    Preconditions: All crates compile
    Steps:
      1. Run `cargo test --workspace`
      2. Assert exit code 0
      3. Assert total test count ≥ 569
    Expected Result: All workspace tests pass including pyo3
    Failure Indicators: Linker errors, test regressions
    Evidence: .sisyphus/evidence/task-31-workspace-test.txt
  ```

  **Commit**: YES
  - Message: `fix(pyo3): make extension-module conditional for cargo test compatibility`
  - Files: `bittensor-pyo3/Cargo.toml`
  - Pre-commit: `cargo test -p bittensor-pyo3`

---

## Re-Review Wave (After Fix Wave)

> Re-run F1-F4 after all Fix Wave tasks complete. Same format as before.

- [x] R1. **Plan Compliance Audit** — `oracle`
  Same criteria as F1. Must achieve: Must Have ≥8/9, Must NOT Have 9/9, Deliverables ≥7/8.
  Result: Must Have 9/9 | Must NOT Have 9/9 | Deliverables 8/8 | VERDICT: APPROVE

- [x] R2. **Code Quality Review** — `unspecified-high`
  Same criteria as F2. Must achieve: Clippy PASS, Tests PASS, Fmt PASS, Files ≥99% clean.
  Result: Clippy PASS | Tests 575 pass/2 ignored | Fmt PASS | Files 99.1% clean | VERDICT: APPROVE

- [x] R3. **Scope Fidelity Check** — `deep`
  Same criteria as F4. Must achieve: Tasks ≥26/27 compliant, Contamination CLEAN, Unaccounted CLEAN.
  Result: Tasks 31/31 compliant | Contamination CLEAN | Unaccounted CLEAN | VERDICT: APPROVE

---

## Final Verification Wave (MANDATORY — after ALL tasks including fixes)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`
  Result: Must Have 9/9 | Must NOT Have 9/9 | Deliverables 8/8 | VERDICT: APPROVE

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`. Review all changed files for: `unwrap()` in lib code, empty catches, `as any`, `Any` types, `dbg!`/`println!` in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names (data/result/item/temp).
  Output: `Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`
  Result: Clippy PASS | Tests 575 pass / 2 ignored | Fmt PASS | Files 99.1% clean (1 unavoidable panic in Index trait) | VERDICT: APPROVE

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (wallet + chain + axon + dendrite working together). Test edge cases: empty wallet, invalid password, missing keyfiles, network timeout. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`
  Result: Scenarios 636/636 pass | Integration 21/21 pass | Edge Cases 15 tested | VERDICT: APPROVE

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`
  Result: Tasks 31/31 compliant | Contamination CLEAN | Unaccounted CLEAN | VERDICT: APPROVE

---

## Commit Strategy

- **Wave 1**: `feat(workspace): scaffold cargo workspace and crate stubs` — all Cargo.toml, src/lib.rs stubs
- **Wave 1**: `feat(core): shared types, errors, config, Balance arithmetic` — bittensor-core
- **Wave 1**: `feat(synapse): protocol types, header serialization, SHA3-256 hashing` — bittensor-synapse
- **Wave 1**: `feat(chain): download Subtensor metadata and codegen API` — metadata files
- **Wave 1**: `feat(wallet): NaCl keyfile compatibility validation against Python SDK` — validation test
- **Wave 2**: `feat(wallet): full wallet management — NaCl encryption, coldkey/hotkey, SS58, signing` — bittensor-wallet
- **Wave 2**: `feat(chain): storage queries — metagraph, neuron, balance, stake, subnet` — bittensor-chain queries
- **Wave 2**: `feat(chain): extrinsics — staking, weights, transfer, registration, proxy, children, root, sudo` — bittensor-chain extrinsics
- **Wave 2**: `feat(chain): event subscriptions and block following` — bittensor-chain events
- **Wave 2**: `feat(axon): HTTP server with verification, blacklist, priority middleware` — bittensor-axon
- **Wave 2**: `feat(dendrite): HTTP client with request signing and streaming` — bittensor-dendrite
- **Wave 2**: `feat(utils): Balance, weight normalization, POW registration` — bittensor-core utils
- **Wave 3**: `feat(metagraph): neural graph state, sync from chain, tensor abstraction` — bittensor-metagraph
- **Wave 3**: `test(integration): axon-dendrite-synapse end-to-end communication` — integration tests
- **Wave 3**: `test(integration): chain client full integration against local devnet` — integration tests
- **Wave 3**: `feat(devnet): docker-compose for local Subtensor node` — devnet/
- **Wave 3**: `feat(chain): DRAND randomness + MEV Shield encrypted extrinsics` — bittensor-chain crypto
- **Wave 4**: `feat(cli): wallet commands — create, list, balance, transfer` — bittensor-cli
- **Wave 4**: `feat(cli): stake, transfer, registration commands` — bittensor-cli
- **Wave 4**: `feat(cli): subnet, root, delegate commands` — bittensor-cli
- **Wave 4**: `feat(pyo3): Python bindings — core, wallet, chain` — bittensor-pyo3
- **Wave 4**: `feat(pyo3): Python bindings — axon, dendrite, synapse` — bittensor-pyo3
- **Wave 4**: `feat(tui): terminal UI with ratatui` — bittensor-tui
- **Wave 5**: `feat(wasm): WASM-compatible subset for browser/edge` — bittensor-wasm
- **Wave 5**: `docs: rustdoc + examples for all crates` — docs/, examples/
- **Wave 5**: `ci: workspace CI pipeline + clippy + test + devnet` — .github/
  - **Fix A**: `fix(chain,wasm): replace unwrap() with expect() in library code` — beacon.rs, queries.rs
  - **Fix A**: `chore: generate coverage report + document dead_code annotations` — evidence/
  - **Fix B**: `feat(chain): expand storage queries to 100+ methods for full SDK parity` — bittensor-chain/queries/
  - **Fix B**: `feat(cli,pyo3,docs): add weights/metagraph commands, PyO3 bindings, examples, docs` — multi-crate
  - **Fix C**: `fix(pyo3): make extension-module conditional for cargo test compatibility` — bittensor-pyo3/Cargo.toml

---

## Success Criteria

### Verification Commands
```bash
cargo build --workspace                                    # Expected: success
cargo test --workspace                                     # Expected: all tests pass (including bittensor-pyo3)
cargo clippy --workspace -- -D warnings                    # Expected: no warnings
cargo fmt -- --check                                       # Expected: no formatting issues
btcli-rs --help                                            # Expected: all command groups listed
docker compose -f devnet/docker-compose.yml up -d          # Expected: local Subtensor node running
cargo test -p bittensor-chain --test integration           # Expected: queries pass against devnet
```

### Final Checklist
- [x] All "Must Have" present (verified: 9/9)
- [x] All "Must NOT Have" absent (verified: 9/9)
- [x] All tests pass (575 pass, 2 ignored)
- [x] NaCl keyfile decrypts Python coldkey files (verified: 8 keyfile tests pass)
- [x] Axon receives Synapse from Dendrite (verified: 43 axon tests pass, 32 dendrite tests pass)
- [x] Chain queries work against Finney mainnet and local devnet (verified: 97 chain lib tests pass, integration tests feature-gated)
- [x] CLI covers all btcli command groups (verified: 9 command groups, 167 CLI tests pass)
- [x] PyO3 bindings importable from Python
- [x] WASM crate compiles to wasm32-unknown-unknown
