extern crate recommendation_service;

use recommendation_service::recommendation::User;

use recommendation_service::elastic::ops::ElasticOperator;
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

    debug!("{:?}", users);

    let transport = Transport::static_node(vec!["http://localhost:9200"]).unwrap();
    let client = Elasticsearch::new(transport);
    let elastic_operator = ElasticOperator::new(client);
    let service = MainRecommendactionService::new(elastic_operator).await;
    // TODO Implement bulk load, can't because one node
    service.new_users(users).await;
}
