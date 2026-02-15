use crate::chain::BittensorClient;
use crate::metagraph::Metagraph;
use crate::types::MetagraphInfo;
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get metagraph information for a subnet
pub async fn get_metagraph_info(client: &BittensorClient, netuid: u16) -> Result<Metagraph> {
    crate::metagraph::sync::sync_metagraph(client, netuid).await
}

/// Get all metagraphs information
pub async fn get_all_metagraphs_info(client: &BittensorClient) -> Result<Vec<Metagraph>> {
    use crate::queries::subnets::all_subnets;

    let netuids = all_subnets(client).await?;
    let mut metagraphs = Vec::new();

    for subnet in netuids {
        if let Ok(metagraph) = get_metagraph_info(client, subnet.netuid).await {
            metagraphs.push(metagraph);
        }
    }

    Ok(metagraphs)
}

/// Get comprehensive MetagraphInfo for a subnet
/// Populates all fields including neuron arrays, parameters, pool data, and emissions
pub async fn get_metagraph_info_full(
    client: &BittensorClient,
    netuid: u16,
) -> Result<MetagraphInfo> {
    let mut info = MetagraphInfo::new(netuid);

    info.block = client
        .block_number()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let n_key = vec![Value::u128(netuid as u128)];
    let n_value = client
        .storage_with_keys(SUBTENSOR_MODULE, "SubnetworkN", n_key.clone())
        .await?;
    let n: u64 = n_value
        .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
        .unwrap_or(0);

    info.num_uids = n;

    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "MaxAllowedUids",
            vec![Value::u128(netuid as u128)],
        )
        .await?
    {
        info.max_uids = crate::utils::decoders::decode_u64(&val).unwrap_or(0);
    }

    info.tempo = crate::queries::subnets::tempo(client, netuid)
        .await?
        .unwrap_or(0);
    info.blocks_since_last_step = crate::queries::subnets::blocks_since_last_step(client, netuid)
        .await?
        .unwrap_or(0);
    info.last_step = info.block.saturating_sub(info.blocks_since_last_step);

    if let Some(owner_ck) = crate::queries::subnets::get_subnet_owner(client, netuid).await? {
        info.owner_coldkey = crate::utils::ss58::encode_ss58(&owner_ck);
    }
    if let Some(owner_hk) = crate::queries::subnets::subnet_owner_hotkey(client, netuid).await? {
        info.owner_hotkey = crate::utils::ss58::encode_ss58(&owner_hk);
    }

    let neurons = crate::queries::neurons::neurons(client, netuid, None).await?;
    let mut sorted_neurons = neurons;
    sorted_neurons.sort_by_key(|n| n.uid);

    for neuron in &sorted_neurons {
        info.hotkeys
            .push(crate::utils::ss58::encode_ss58(&neuron.hotkey));
        info.coldkeys
            .push(crate::utils::ss58::encode_ss58(&neuron.coldkey));
        info.active.push(neuron.active);
        info.validator_permit.push(neuron.validator_permit);
        info.pruning_score.push(neuron.pruning_score as f64);
        info.last_update.push(neuron.last_update);
        info.emission.push(neuron.emission.as_u128() as f64);
        info.incentive.push(neuron.incentive);
        info.consensus.push(neuron.consensus);
        info.trust.push(neuron.trust);
        info.validator_trust.push(neuron.validator_trust);
        info.dividends.push(neuron.dividends);
        info.rank.push(neuron.rank);
        info.alpha_stake.push(neuron.stake.as_u128() as f64);
        info.tao_stake.push(neuron.root_stake.as_u128() as f64);
        info.total_stake.push(neuron.total_stake.as_u128() as f64);
    }

    info.hparams.tempo = info.tempo;
    if let Ok(hp) =
        crate::queries::hyperparameters::get_subnet_hyperparameters(client, netuid).await
    {
        info.hparams.immunity_period = hp.immunity_period as u64;
        info.hparams.min_allowed_weights = hp.min_allowed_weights as u64;
        info.hparams.max_weights_limit = hp.max_weights_limit as u64;
        info.hparams.max_validators = hp.max_validators as u64;
        info.hparams.rho = hp.rho as u64;
        info.hparams.kappa = hp.kappa as u64;
        info.hparams.difficulty = hp.difficulty as u128;
        info.hparams.weights_rate_limit = hp.weights_rate_limit;
        info.hparams.weights_version = hp.weights_version;
        info.hparams.registration_allowed = hp.registration_allowed;
        info.hparams.serving_rate_limit = hp.serving_rate_limit;
        info.hparams.commit_reveal_weights_enabled = hp.commit_reveal_weights_enabled;
        info.hparams.liquid_alpha_enabled = hp.liquid_alpha_enabled;
    }

    if let Ok(dynamic) = crate::queries::subnets::get_dynamic_info(client, netuid).await {
        info.pool.alpha_in = dynamic.alpha_in as f64;
        info.pool.alpha_out = dynamic.alpha_out as f64;
        info.pool.tao_in = dynamic.tao_in as f64;
        info.pool.subnet_volume = dynamic.subnet_volume as f64;
        info.pool.moving_price = dynamic.moving_price as f64;

        info.emissions.alpha_out_emission = dynamic.alpha_out_emission as f64;
        info.emissions.tao_in_emission = dynamic.tao_in_emission as f64;
        info.emissions.subnet_emission = dynamic.emission_value as f64;
        info.emissions.pending_alpha_emission = dynamic.pending_emission as f64;
    }

    Ok(info)
}
