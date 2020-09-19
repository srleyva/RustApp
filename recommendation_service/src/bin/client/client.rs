extern crate recommendation_service;

use env_logger::init;
use log::info;
use tonic::Request;

use recommendation_service::recommendation::{
    recommendation_service_client::RecommendationServiceClient, Gender, GetQueueRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    let channel = tonic::transport::Channel::from_static("http://localhost:3030")
        .connect()
        .await
        .unwrap();
    let mut client = RecommendationServiceClient::new(channel);
    let request = Request::new(GetQueueRequest {
        uid: "541fe12b-e1ab-45e6-a558-d2ddaac310b6".to_string(),
        longitude: -132.8896,
        latitude: 67.7974,
        radius: 50,
        age_range: vec![21, 30],
        gender: Gender::Female as i32,
    });
    let mut result = client.get_queue(request).await.unwrap().into_inner();

    while let Some(user) = result.message().await.unwrap() {
        info!("Got User: {:?}", user);
    }
    Ok(())
}
