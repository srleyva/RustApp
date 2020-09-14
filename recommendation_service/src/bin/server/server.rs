extern crate recommendation_service;

use elasticsearch::{http::transport::Transport, Elasticsearch};
use env_logger::init;
use log::info;
use recommendation_service::elastic::ops::ElasticOperator;
use recommendation_service::recommendation::recommendation_service_server::RecommendationServiceServer;
use recommendation_service::service::MainRecommendactionService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    info!("Starting recommendation-Service");

    let elastic_operator = ElasticOperator::new(Elasticsearch::new(
        Transport::single_node("http://localhost:9200").unwrap(),
    ));

    let rec_service =
        RecommendationServiceServer::new(MainRecommendactionService::new(elastic_operator).await);

    let addr = "0.0.0.0:3030".parse().unwrap();
    info!("Server listening on {}", addr);
    Server::builder()
        .add_service(rec_service)
        .serve(addr)
        .await?;
    Ok(())
}
