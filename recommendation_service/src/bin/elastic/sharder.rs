extern crate recommendation_service;

use recommendation_service::elasticsearch::ops::{
    build_geoshard_mapping_index,
    build_geosharded_indices
};

fn main() {
    env_logger::init();
    let client = SyncClient::builder().sniff_nodes("http://localhost:9200").build().unwrap();
    client.ping().send().unwrap();
    build_geoshard_mapping_index(es_client: SyncClient, shards: &Shards)
}