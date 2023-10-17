use crate::scarb::ForkTarget;
use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use cheatnet::forking::state::ForkStateReader;
use conversions::StarknetConversions;
use starknet::core::types::BlockTag::Latest;
use starknet::core::types::MaybePendingBlockWithTxHashes;
use starknet::core::types::{BlockId, BlockTag};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use test_collector::ForkConfig;
use tokio::runtime::Runtime;
use url::Url;

pub fn get_fork_state_reader(
    workspace_root: &Utf8PathBuf,
    fork_targets: &[ForkTarget],
    fork_config: &Option<ForkConfig>,
) -> Result<Option<ForkStateReader>> {
    match &fork_config {
        Some(ForkConfig::Params(url, mut block_id)) => {
            if let BlockId::Tag(Latest) = block_id {
                block_id = get_latest_block_number(url)?;
            }
            Ok(Some(ForkStateReader::new(
                url,
                block_id,
                Some(workspace_root.join(".snfoundry_cache").as_ref()),
            )))
        }
        Some(ForkConfig::Id(name)) => {
            find_params_and_build_fork_state_reader(workspace_root, fork_targets, name)
        }
        _ => Ok(None),
    }
}

fn find_params_and_build_fork_state_reader(
    workspace_root: &Utf8PathBuf,
    fork_targets: &[ForkTarget],
    fork_alias: &str,
) -> Result<Option<ForkStateReader>> {
    if let Some(fork) = fork_targets.iter().find(|fork| fork.name == fork_alias) {
        let block_id = fork
            .block_id
            .iter()
            .map(|(id_type, value)| match id_type.as_str() {
                "number" => Some(BlockId::Number(value.parse().unwrap())),
                "hash" => Some(BlockId::Hash(value.to_field_element())),
                "tag" => match value.as_str() {
                    "Latest" => Some(BlockId::Tag(BlockTag::Latest)),
                    "Pending" => Some(BlockId::Tag(BlockTag::Pending)),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        let [Some(mut block_id)] = block_id[..] else {
            return Ok(None);
        };

        if let BlockId::Tag(Latest) = block_id {
            block_id = get_latest_block_number(&fork.url)?;
        }
        return Ok(Some(ForkStateReader::new(
            &fork.url,
            block_id,
            Some(workspace_root.join(".snfoundry_cache").as_ref()),
        )));
    }

    Ok(None)
}

fn get_latest_block_number(url: &str) -> Result<BlockId> {
    let client = JsonRpcClient::new(HttpTransport::new(Url::parse(url).unwrap()));
    let runtime = Runtime::new().expect("Could not instantiate Runtime");

    match runtime.block_on(client.get_block_with_tx_hashes(BlockId::Tag(Latest))) {
        Ok(MaybePendingBlockWithTxHashes::Block(block)) => Ok(BlockId::Number(block.block_number)),
        _ => Err(anyhow!("Could not get the latest block number".to_string())),
    }
}
