use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::balance::Balance;

/// Endpoint metadata for a neuron's axon (server-side) on the network.
///
/// The `ip` field is stored as a packed `u64` (e.g. `2130706433` = `127.0.0.1`).
/// `ip_type` is `4` for IPv4, `6` for IPv6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct AxonInfo {
    /// Packed IP address as u64 (network byte order).
    pub ip: u64,
    /// TCP port the axon listens on.
    pub port: u16,
    /// IP protocol version: `4` = IPv4, `6` = IPv6.
    pub ip_type: u8,
    /// Application-layer protocol identifier.
    pub protocol: u8,
    /// Axon protocol version (incremented on breaking changes).
    pub version: u32,
    /// SS58-encoded hotkey (signing key) of the neuron.
    pub hotkey: String,
    /// SS58-encoded coldkey (owner key) of the neuron.
    pub coldkey: String,
}

/// Prometheus metrics endpoint for a neuron.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct PrometheusInfo {
    /// Packed IP address as u64.
    pub ip: u64,
    /// Prometheus scrape port.
    pub port: u16,
    /// Metrics protocol version.
    pub version: u32,
    /// Block at which this info was last updated.
    pub block: u64,
}

/// Full neuron metadata, including weight/bond vectors and per-nominator stake.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct NeuronInfo {
    /// Unique neuron UID within the subnet.
    pub uid: u16,
    /// Subnet ID this neuron belongs to.
    pub netuid: u16,
    /// Whether the neuron is actively serving.
    pub active: bool,
    /// Total stake (delegated + self-stake) in rao.
    pub stake: Balance,
    /// Rank score (0–65535 scaled).
    pub rank: u16,
    /// Trust score (0–65535 scaled).
    pub trust: u16,
    /// Consensus score (0–65535 scaled).
    pub consensus: u16,
    /// Incentive score (0–65535 scaled).
    pub incentive: u16,
    /// Dividend earned (0–65535 scaled).
    pub dividend: u16,
    /// Emission in rao per tempo.
    pub emission: u64,
    /// Prometheus endpoint, if registered.
    pub prometheus_info: Option<PrometheusInfo>,
    /// Axon endpoint, if registered.
    pub axon_info: Option<AxonInfo>,
    /// SS58-encoded hotkey.
    pub hotkey: String,
    /// SS58-encoded coldkey.
    pub coldkey: String,
    /// Block number of the neuron's last weight update.
    pub last_update: u64,
    /// Validator trust score (0–65535 scaled).
    pub validator_trust: u16,
    /// Weight vector (u16 per peer, scaled by `max_weight_limit`).
    pub weights: Vec<u16>,
    /// Bond vector (u16 per peer).
    pub bonds: Vec<u16>,
    /// Per-nominator stake: `(ss58_address, balance)` pairs.
    pub stake_dict: Vec<(String, Balance)>,
}

/// Lightweight neuron metadata (no weight/bond vectors), used for list queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct NeuronInfoLite {
    /// Unique neuron UID within the subnet.
    pub uid: u16,
    /// SS58-encoded hotkey.
    pub hotkey: String,
    /// SS58-encoded coldkey.
    pub coldkey: String,
    /// Whether the neuron is actively serving.
    pub active: bool,
    /// Total stake in rao.
    pub stake: Balance,
    /// Rank score (0–65535 scaled).
    pub rank: u16,
    /// Trust score (0–65535 scaled).
    pub trust: u16,
    /// Consensus score (0–65535 scaled).
    pub consensus: u16,
    /// Incentive score (0–65535 scaled).
    pub incentive: u16,
}

/// Stake entry for a single hotkey/coldkey pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct StakeInfo {
    /// SS58-encoded hotkey that holds the stake.
    pub hotkey: String,
    /// SS58-encoded coldkey that owns the stake.
    pub coldkey: String,
    /// Stake amount in rao.
    pub stake: Balance,
}

/// Delegate information including nominators, take, and registration list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct DelegateInfo {
    /// SS58 address of the delegate.
    pub delegate_ss58: String,
    /// SS58-encoded hotkey of the delegate.
    pub delegate_hotkey: String,
    /// Total stake held by the delegate (self-stake + nominations).
    pub total_stake: Balance,
    /// Nominator list: `(ss58_address, balance)` pairs.
    pub nominators: Vec<(String, Balance)>,
    /// Hotkey that owns this delegate (may differ from delegate hotkey).
    pub owner_hotkey: String,
    /// Delegate take as parts-per-ten-thousand (e.g. 1800 = 18%).
    pub take: u16,
    /// SS58 address of the owner.
    pub owner_ss58: String,
    /// Subnet UIDs where the delegate is registered.
    pub registrations: Vec<u16>,
    /// Subnet UIDs where the delegate has validator permission.
    pub validator_permits: Vec<u16>,
}

/// Subnet metadata including owner and tempo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct SubnetInfo {
    /// Subnet unique identifier.
    pub netuid: u16,
    /// Human-readable subnet name.
    pub name: String,
    /// SS58-encoded hotkey of the subnet owner.
    pub owner_hotkey: String,
    /// Tempo: blocks between weight-setting rounds.
    pub tempo: u16,
    /// Optional chain-level identity (name + symbol).
    pub subnet_identity: Option<ChainIdentity>,
    /// Highest assigned UID in the subnet.
    pub maximum_uid: u16,
    /// Modality type (0 = text, 1 = image, 2 = audio, 3 = video).
    pub modality: u8,
    /// Network UID (for cross-chain references).
    pub network_uid: u16,
}

/// Subnet hyperparameters controlling incentive distribution and registration.
///
/// All `u16` score fields use a 65535-based scale. `burn`/`min_burn`/`max_burn`
/// are in rao.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct SubnetHyperparameters {
    /// Rho: overall incentive power.
    pub rho: u16,
    /// Kappa: temperature for sigmoid-based trust calculation.
    pub kappa: u16,
    /// POW difficulty for registration.
    pub difficulty: u32,
    /// Burn cost to register (rao).
    pub burn: u64,
    /// Immunity period as a ratio of subnet size.
    pub immunity_ratio: u16,
    /// Minimum registration burn (rao).
    pub min_burn: u64,
    /// Maximum registration burn (rao).
    pub max_burn: u64,
    /// Minimum blocks between weight-setting for a validator.
    pub weights_rate_limit: u64,
    /// Weight version identifier.
    pub weights_version: u16,
    /// Minimum stake required to set weights (rao).
    pub weights_min_stake: u64,
    /// Maximum total weight a neuron can distribute (u16 scale).
    pub max_weight_limit: u16,
    /// Scaling law exponent.
    pub scaling_law_power: u16,
    /// Current number of neurons in the subnet.
    pub subnetwork_n: u16,
    /// Maximum allowed neurons in the subnet.
    pub max_n: u16,
    /// Blocks elapsed since last weight adjustment step.
    pub blocks_since_last_step: u64,
    /// Blocks between weight adjustment steps.
    pub tempo: u16,
    /// Alpha for exponential moving average adjustments.
    pub adjustment_alpha: u64,
    /// Blocks between adjustment intervals.
    pub adjustment_interval: u16,
    /// Bond moving average denominator.
    pub bonds_moving_avg: u64,
    /// Upper bound for liquid alpha.
    pub alpha_high: u16,
    /// Lower bound for liquid alpha.
    pub alpha_low: u16,
    /// Whether liquid alpha is enabled for this subnet.
    pub liquid_alpha_enabled: bool,
}

/// On-chain identity for a subnet (name + symbol).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct ChainIdentity {
    /// Subnet this identity belongs to.
    pub netuid: u16,
    /// Human-readable subnet name.
    pub name: String,
    /// Ticker symbol (e.g. "TAO", "COMPUTE").
    pub symbol: String,
}

/// Committed weight hash for a subnet, used in commit-reveal weight setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct WeightCommitInfo {
    /// Hotkey that submitted the commit.
    pub hotkey: String,
    /// Opaque commit bytes (hash of the weight vector + salt).
    pub commit: Vec<u8>,
    /// Block at which the commit can be revealed.
    pub reveal_round: u64,
    /// Subnet the commit targets.
    pub netuid: u16,
}

/// Snapshot of a subnet's state at a given block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct SubnetState {
    /// Subnet unique identifier.
    pub netuid: u16,
    /// Block number of this snapshot.
    pub block: u64,
    /// Total stake in the subnet.
    pub stake: Balance,
    /// Total emission in rao for this subnet.
    pub emission: u64,
    /// Whether the subnet is active.
    pub active: bool,
}

/// Governance proposal with vote tallies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct ProposalVoteData {
    /// Proposal index.
    pub index: u32,
    /// Votes required to pass.
    pub threshold: u32,
    /// SS58 addresses that voted aye.
    pub ayes: Vec<String>,
    /// SS58 addresses that voted nay.
    pub nays: Vec<String>,
    /// Block at which voting ends.
    pub end: u64,
}

/// Aggregate statistics for a subnet metagraph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct MetagraphInfo {
    /// Subnet unique identifier.
    pub netuid: u16,
    /// Block at which this info was captured.
    pub block: u64,
    /// Number of neurons in the subnet.
    pub n: u16,
    /// Total stake across all neurons.
    pub stake: Balance,
    /// Total issuance for the subnet.
    pub total_issuance: Balance,
    /// Sum of all weight values.
    pub total_weight: u64,
    /// Sum of all bond values.
    pub total_bond: u64,
}

/// A neuron's TLS certificate registered on-chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct NeuronCertificate {
    /// SS58-encoded hotkey this certificate belongs to.
    pub hotkey: String,
    /// DER-encoded TLS certificate bytes.
    pub certificate: Vec<u8>,
    /// Block at which the certificate was registered.
    pub block: u64,
}

/// Moving average registration price for a subnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct MovingPriceInfo {
    /// Subnet unique identifier.
    pub netuid: u16,
    /// Current moving-average price in rao.
    pub price: Balance,
    /// Block at which this price was computed.
    pub block: u64,
}

/// Scheduled weight-setting event for a subnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleInfo {
    /// Block at which the event fires.
    pub block: u64,
    /// Subnet unique identifier.
    pub netuid: u16,
    /// Tempo (blocks between events) for this subnet.
    pub tempo: u16,
}

/// A balance transfer record on chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct TransferInfo {
    /// SS58 address of the sender.
    pub from: String,
    /// SS58 address of the receiver.
    pub to: String,
    /// Amount transferred in rao.
    pub amount: Balance,
    /// Block at which the transfer was included.
    pub block: u64,
}

/// A stake move (add/remove) record on chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct StakeTransferInfo {
    /// SS58-encoded hotkey involved in the stake.
    pub hotkey: String,
    /// SS58-encoded coldkey involved in the stake.
    pub coldkey: String,
    /// Amount staked/unstaked in rao.
    pub amount: Balance,
    /// Block at which the stake operation was included.
    pub block: u64,
    /// Module key for childkey take, if applicable.
    pub module_key: Option<String>,
}

/// Delegate take change record on chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct DelegateTakeInfo {
    /// SS58-encoded hotkey of the delegate.
    pub hotkey: String,
    /// New take value (parts-per-ten-thousand).
    pub take: u16,
    /// Block at which the take change was applied.
    pub block: u64,
}

/// Neuron registration record (burn-based registration).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationInfo {
    /// Subnet the neuron registered on.
    pub netuid: u16,
    /// SS58-encoded hotkey of the registered neuron.
    pub hotkey: String,
    /// Block at which registration occurred.
    pub block: u64,
    /// Burn cost paid for registration.
    pub burn: Balance,
}

/// Audit score record for a neuron.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct AuditInfo {
    /// SS58-encoded hotkey of the audited neuron.
    pub hotkey: String,
    /// SS58-encoded coldkey of the audited neuron.
    pub coldkey: String,
    /// Subnet the neuron belongs to.
    pub netuid: u16,
    /// Block at which the audit was recorded.
    pub block: u64,
    /// Audit score (0–65535 scaled).
    pub score: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scale_roundtrip<T: Encode + Decode + PartialEq + std::fmt::Debug>(value: &T) {
        let encoded = value.encode();
        let decoded: T = Decode::decode(&mut &encoded[..]).expect("decode");
        assert_eq!(*value, decoded);
    }

    fn json_roundtrip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
        value: &T,
    ) {
        let json = serde_json::to_string(value).expect("serialize");
        let decoded: T = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*value, decoded);
    }

    #[test]
    fn axon_info_scale_roundtrip() {
        let info = AxonInfo {
            ip: 2130706433,
            port: 8090,
            ip_type: 4,
            protocol: 0,
            version: 1,
            hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
            coldkey: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".into(),
        };
        scale_roundtrip(&info);
    }

    #[test]
    fn axon_info_json_roundtrip() {
        let info = AxonInfo {
            ip: 0,
            port: 0,
            ip_type: 0,
            protocol: 0,
            version: 0,
            hotkey: String::new(),
            coldkey: String::new(),
        };
        json_roundtrip(&info);
    }

    #[test]
    fn neuron_info_lite_scale_roundtrip() {
        let info = NeuronInfoLite {
            uid: 42,
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            active: true,
            stake: Balance::from_tao(100.0),
            rank: 10,
            trust: 5,
            consensus: 3,
            incentive: 7,
        };
        scale_roundtrip(&info);
    }

    #[test]
    fn delegate_info_json_roundtrip() {
        let info = DelegateInfo {
            delegate_ss58: "5Test".into(),
            delegate_hotkey: "hk".into(),
            total_stake: Balance::from_tao(500.0),
            nominators: vec![("nom1".into(), Balance::from_tao(250.0))],
            owner_hotkey: "owner".into(),
            take: 18,
            owner_ss58: "5Owner".into(),
            registrations: vec![1, 3],
            validator_permits: vec![1],
        };
        json_roundtrip(&info);
    }

    #[test]
    fn subnet_hyperparams_scale_roundtrip() {
        let hp = SubnetHyperparameters {
            rho: 10,
            kappa: 32768,
            difficulty: 1000000,
            burn: 1000,
            immunity_ratio: 25,
            min_burn: 500,
            max_burn: 2000,
            weights_rate_limit: 100,
            weights_version: 0,
            weights_min_stake: 0,
            max_weight_limit: 1000,
            scaling_law_power: 50,
            subnetwork_n: 256,
            max_n: 1024,
            blocks_since_last_step: 0,
            tempo: 360,
            adjustment_alpha: 500,
            adjustment_interval: 100,
            bonds_moving_avg: 1000,
            alpha_high: 58,
            alpha_low: 60,
            liquid_alpha_enabled: false,
        };
        scale_roundtrip(&hp);
    }

    #[test]
    fn chain_identity_json_roundtrip() {
        let ci = ChainIdentity { netuid: 1, name: "compute".into(), symbol: "COMPUTE".into() };
        json_roundtrip(&ci);
    }

    #[test]
    fn weight_commit_scale_roundtrip() {
        let wci = WeightCommitInfo {
            hotkey: "hk".into(),
            commit: vec![1, 2, 3, 4],
            reveal_round: 1000,
            netuid: 1,
        };
        scale_roundtrip(&wci);
    }

    #[test]
    fn stake_info_with_balance() {
        let si = StakeInfo {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            stake: Balance::from_rao(1_500_000_000),
        };
        assert_eq!(si.stake.to_rao(), 1_500_000_000);
        scale_roundtrip(&si);
    }

    #[test]
    fn prometheus_info_scale_roundtrip() {
        let pi = PrometheusInfo { ip: 16777343, port: 9100, version: 1, block: 12345 };
        scale_roundtrip(&pi);
    }

    #[test]
    fn subnet_info_scale_roundtrip() {
        let si = SubnetInfo {
            netuid: 1,
            name: "root".into(),
            owner_hotkey: "owner".into(),
            tempo: 100,
            subnet_identity: None,
            maximum_uid: 256,
            modality: 0,
            network_uid: 1,
        };
        scale_roundtrip(&si);
    }

    #[test]
    fn proposal_vote_data_json_roundtrip() {
        let pvd = ProposalVoteData {
            index: 1,
            threshold: 100,
            ayes: vec!["a".into()],
            nays: vec!["b".into()],
            end: 50000,
        };
        json_roundtrip(&pvd);
    }

    #[test]
    fn neuron_certificate_scale_roundtrip() {
        let nc = NeuronCertificate { hotkey: "hk".into(), certificate: vec![0u8; 32], block: 999 };
        scale_roundtrip(&nc);
    }

    #[test]
    fn registration_info_json_roundtrip() {
        let ri = RegistrationInfo {
            netuid: 1,
            hotkey: "hk".into(),
            block: 100,
            burn: Balance::from_tao(1.0),
        };
        json_roundtrip(&ri);
    }

    #[test]
    fn audit_info_scale_roundtrip() {
        let ai = AuditInfo {
            hotkey: "hk".into(),
            coldkey: "ck".into(),
            netuid: 1,
            block: 200,
            score: 85,
        };
        scale_roundtrip(&ai);
    }

    #[test]
    fn subnet_state_json_roundtrip() {
        let ss = SubnetState {
            netuid: 1,
            block: 100,
            stake: Balance::from_tao(50.0),
            emission: 1000,
            active: true,
        };
        json_roundtrip(&ss);
    }
}
