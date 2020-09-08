use super::indices::GeoshardMappingIndex;
use elastic::prelude::*;
use elastic::client::{SyncClient};
use log::info;


use super::super::location::sharding::Shards;

pub fn build_geoshard_mapping_index(es_client: SyncClient, shards: &Shards) {
    info!("Building Geoshard Mapping Index: {}", GeoshardMappingIndex::name())
}

pub fn build_geosharded_indices(es_client: SyncClient, shards: &Shards) {
    info!("Building shards from level {} index", 7);
    for geoshard in &shards.shards {
        info!("Building index from geoshard: {}", geoshard.name);
        let shard_index = index(geoshard.name.clone());
        let response = es_client.index(shard_index).create().send().unwrap();
        assert!(response.acknowledged());
    }
}