use super::indices::{
    GeoShardMappingIndex,
    UserIndex,
};
use elastic::prelude::*;
use elastic::client::{SyncClient};
use log::info;


use super::super::location::sharding::GeoShard;

// TODO
pub fn build_geoshard_mapping_index(_es_client: SyncClient) {
    info!("Building Geoshard Mapping Index: {}", GeoShardMappingIndex::name())
}

pub fn build_geosharded_indices(es_client: SyncClient, shards: &Vec<GeoShard>) {
    info!("Building shards from level {} index", 7);
    for geoshard in shards {
        info!("Building user index from geoshard: {}", geoshard.name);
        let user_index = UserIndex::from(geoshard);
        let shard_index = index(user_index.name());
        let response = es_client.index(shard_index).create().send().unwrap();
        assert!(response.acknowledged());
    }
}