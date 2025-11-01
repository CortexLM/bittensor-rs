use crate::chain::BittensorClient;
use crate::metagraph::Metagraph;
use anyhow::Result;

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
