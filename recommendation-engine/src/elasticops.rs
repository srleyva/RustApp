use elastic::client::{SyncClient, SyncClientBuilder};
use elastic::client::requests::params::index;
use elastic::http::header::Authorization;
use log::info;
use env_logger::init;

use location::Shards;

mod location;

fn main() {
    env_logger::init();
    let builder = SyncClientBuilder::new()
    .base_url("http://localhost:9200")
    .params(|p| p
        .url_param("pretty", true)
        .header(Authorization("let me in".to_owned())));
    let client = builder.build().unwrap();
    info!("Connecting to ES Cluster at {}", "http://localhost:9200");
    client.ping().send().unwrap();
    build_index_from_geoshard(client);
}

fn build_index_from_geoshard(es_client: SyncClient) {
    info!("Building shards from level {} index", 7);
    let shard = Shards::new(7);

    for geoshard in shard.shards {
        info!("Building index from geoshard: {}", geoshard.name);
        let shard_index = index(geoshard.name);
        let response = es_client.index_create(shard_index).send().unwrap();
        assert!(response.acknowledged());
    }
}