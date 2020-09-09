extern crate recommendation_service;

use recommendation_service::location::sharding::GeoshardBuilder;
use recommendation_service::elastic::ops::{
    build_geoshard_mapping_index,
    build_geosharded_indices,
};

use elasticsearch::{
    http::transport::Transport,
    Elasticsearch,
};

use env_logger::init;
use log::{
    info
};
use futures::join;

#[tokio::main]
async fn main() {
    init();
    info!("geosharder running");
    let shards = GeoshardBuilder::new(4).into_geosharded_list();

    info!("ES @ http://localhost:9200");
    let transport = Transport::single_node("http://localhost:9200").unwrap();
    let client = Elasticsearch::new(transport);

    let create_mapping_ftr = build_geoshard_mapping_index(&client, &shards);
    let create_indices_ftr = build_geosharded_indices(&client, &shards);

    join!(create_indices_ftr, create_mapping_ftr);
}