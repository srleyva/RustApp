extern crate recommendation_service;

use tonic::{
    Request, 
};
use log::{
    info
};
use env_logger;

use recommendation_service::recommendation::{
    GetQueueRequest,
    recommendation_service_client::RecommendationServiceClient
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let channel = tonic::transport::Channel::from_static("http://localhost:3030")
        .connect()
        .await.unwrap();
    let mut client = RecommendationServiceClient::new(channel);
    
    let request = Request::new(GetQueueRequest {
        longitude: 47.6062,
        latitude: 122.3321,
        radius: 20
    });
   
    let mut result = client.get_queue(request).await.unwrap().into_inner();

    while let Some(user) = result.message().await.unwrap() {
        info!("Got User: {:?}", user);
    }
    Ok(())
}