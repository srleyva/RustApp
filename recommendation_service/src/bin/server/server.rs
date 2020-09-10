extern crate recommendation_service;

use env_logger::init;
use log::{info};
use env_logger;
use tonic::{
    transport::Server, 
};
use recommendation_service::recommendation::recommendation_service_server::{
    RecommendationServiceServer
};

use recommendation_service::service::MainRecommendactionService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    init();
    info!("Starting recommendation-Service");
    let addr = "0.0.0.0:3030".parse().unwrap();

    info!("Server listening on {}", addr);

    let rec_service = RecommendationServiceServer::new(
        MainRecommendactionService::default());

    Server::builder()
        .add_service(rec_service)
        .serve(addr)
        .await?;
    Ok(())
}