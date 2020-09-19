extern crate recommendation_service;

use recommendation_service::recommendation::User;

use recommendation_service::elastic::ops::{
    build_geoshard_mapping_index, build_geosharded_indices,
};

use futures::join;
use recommendation_service::elastic::ops::ElasticOperator;
use recommendation_service::location::sharding::GeoshardBuilder;
use recommendation_service::service::MainRecommendactionService;

use elasticsearch::{http::transport::Transport, Elasticsearch};

use env_logger::init;
use log::{debug, info};

use serde_json::{from_str, Value};
use std::fs;

#[tokio::main]
async fn main() {
    init();
    info!("loading users");
    let users = fs::read_to_string("./seed/seed-data.txt").unwrap();
    let users: Value = from_str(users.as_str()).unwrap();

    let users: Vec<User> = users
        .as_array()
        .unwrap()
        .iter()
        .map(|h| serde_json::from_value(h.clone()).unwrap())
        .collect();

    info!("generating Geoshards");
    let shards = GeoshardBuilder::user_count_scorer(7, &users).build();
    debug!("{:?}", shards);

    info!("ES @ http://localhost:9200");
    let transport = Transport::single_node("http://localhost:9200").unwrap();
    let client = Elasticsearch::new(transport);

    info!("Building Geoshard mapping index");
    let create_mapping_ftr = build_geoshard_mapping_index(&client, &shards);

    info!("Building Geoshard Indices");
    let create_indices_ftr = build_geosharded_indices(&client, &shards);

    join!(create_indices_ftr, create_mapping_ftr);

    let elastic_operator = ElasticOperator::new(client);
    let service = MainRecommendactionService::new(elastic_operator).await;
    // TODO Implement bulk load, can't because one node
    service.new_users(users).await;
}
