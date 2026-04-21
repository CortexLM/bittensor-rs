//! Metagraph serialization — save/load metagraph state as JSON files.

use std::path::Path;

use crate::metagraph::Metagraph;
use crate::sync::Result;

/// Serialize a metagraph to a JSON file at `path`.
pub fn save(metagraph: &Metagraph, path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| bittensor_core::error::BittensorError::Validation("invalid path".into()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| bittensor_core::error::BittensorError::Validation(e.to_string()))?;
    let json = serde_json::to_string_pretty(metagraph)
        .map_err(|e| bittensor_core::error::BittensorError::Codec(e.to_string()))?;
    std::fs::write(path, json)
        .map_err(|e| bittensor_core::error::BittensorError::Validation(e.to_string()))?;
    Ok(())
}

/// Deserialize a metagraph from a JSON file at `path`.
pub fn load(path: &Path) -> Result<Metagraph> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| bittensor_core::error::BittensorError::Validation(e.to_string()))?;
    serde_json::from_str(&data)
        .map_err(|e| bittensor_core::error::BittensorError::Codec(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_core::balance::Balance;
    use bittensor_core::types::{AxonInfo, NeuronInfo, PrometheusInfo};
    use tempfile::TempDir;

    fn make_test_metagraph() -> Metagraph {
        let neurons = vec![NeuronInfo {
            uid: 0,
            netuid: 1,
            active: true,
            stake: Balance::from_tao(100.0),
            rank: 5,
            trust: 3,
            consensus: 2,
            incentive: 7,
            dividend: 1,
            emission: 5000,
            prometheus_info: Some(PrometheusInfo {
                ip: 16777343,
                port: 9100,
                version: 1,
                block: 100,
            }),
            axon_info: Some(AxonInfo {
                ip: 2130706433,
                port: 8090,
                ip_type: 4,
                protocol: 0,
                version: 1,
                hotkey: "0xhk0".into(),
                coldkey: "0xck0".into(),
            }),
            hotkey: "0xhk0".into(),
            coldkey: "0xck0".into(),
            last_update: 0,
            validator_trust: 8,
            weights: vec![0, 5],
            bonds: vec![0, 3],
            stake_dict: vec![],
        }];
        Metagraph::from_neurons(1, 1000, &neurons)
    }

    #[test]
    fn test_save_load_roundtrip() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("metagraph.json");
        let mg = make_test_metagraph();

        save(&mg, &path).expect("save");
        let loaded = load(&path).expect("load");

        assert_eq!(loaded.netuid, mg.netuid);
        assert_eq!(loaded.n, mg.n);
        assert_eq!(loaded.uids, mg.uids);
        assert_eq!(loaded.hotkeys, mg.hotkeys);
        assert_eq!(loaded.coldkeys, mg.coldkeys);
        assert_eq!(loaded.active, mg.active);
        assert_eq!(loaded.block, mg.block);
    }

    #[test]
    fn test_save_creates_parent_dirs() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("nested").join("dir").join("mg.json");
        let mg = make_test_metagraph();

        save(&mg, &path).expect("save with nested dirs");
        assert!(path.exists());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load(Path::new("/tmp/__nonexistent_metagraph_test__.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_save_load_preserves_tensors() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("tensors.json");
        let mg = make_test_metagraph();

        save(&mg, &path).expect("save");
        let loaded = load(&path).expect("load");

        assert_eq!(loaded.stake, mg.stake);
        assert_eq!(loaded.ranks, mg.ranks);
        assert_eq!(loaded.trust, mg.trust);
        assert_eq!(loaded.consensus, mg.consensus);
        assert_eq!(loaded.validator_trust, mg.validator_trust);
        assert_eq!(loaded.incentive, mg.incentive);
        assert_eq!(loaded.dividends, mg.dividends);
        assert_eq!(loaded.emission, mg.emission);
        assert_eq!(loaded.weights, mg.weights);
        assert_eq!(loaded.bonds, mg.bonds);
    }

    #[test]
    fn test_save_load_empty_metagraph() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("empty.json");
        let mg = Metagraph::new(42);

        save(&mg, &path).expect("save");
        let loaded = load(&path).expect("load");

        assert_eq!(loaded.n, 0);
        assert_eq!(loaded.netuid, 42);
    }
}
