use anyhow::Result;
use bittensor_rs::chain::BittensorClient;
use bittensor_rs::queries::chain_info;

#[tokio::main]
async fn main() -> Result<()> {
	let rpc = std::env::var("BITTENSOR_RPC").unwrap_or_else(|_| "wss://entrypoint-finney.opentensor.ai:443".to_string());
	let client = BittensorClient::new(rpc).await?;

	let ts = chain_info::get_timestamp(&client).await?;
	let drand = chain_info::last_drand_round(&client).await?;
	let txrl = chain_info::tx_rate_limit(&client).await?;
	println!("timestamp_ms={} last_drand_round={:?} tx_rate_limit={:?}", ts, drand, txrl);
	Ok(())
}
