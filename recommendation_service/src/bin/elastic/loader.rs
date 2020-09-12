extern crate recommendation_service;

use recommendation_service::recommendation::{
    Location,
    User
};

use std::iter::Cycle;
use std::vec::IntoIter;
use recommendation_service::service::MainRecommendactionService;
use recommendation_service::elastic::ops::ElasticOperator;
use recommendation_service::location::sharding::GeoShardSearcher;

use elasticsearch::{
    BulkParts,
    http::{
        Url
    },
    http::transport::{
        Transport,
        ConnectionPool,
        Connection
    },
    Elasticsearch,
};

use env_logger::init;
use log::{
    info,
    debug
};

use std::fs;
use serde_json::{
    from_str,
    Value
};

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
    
    let transport = Transport::single_node("http://localhost:9200").unwrap();
    let client = Elasticsearch::new(transport);
    let elastic_operator = ElasticOperator::new(client);
    
    let service = MainRecommendactionService::new(elastic_operator).await;
    
    // TODO Implement bulk load
    service.new_users(users).await;
}