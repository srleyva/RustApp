use futures::Stream;
use std::pin::Pin;
use tonic::{
    Request, 
    Response, 
    Status
};
use super::recommendation::{
    User,
    Location,

    GetQueueRequest,

    SwipeRequest,
    SwipeResponse,
    recommendation_service_server::{
        RecommendationService,
    }
};

#[derive(Default)]
pub struct MainRecommendactionService {

}

type UserStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send + Sync>>;

#[tonic::async_trait]
impl RecommendationService for MainRecommendactionService {
    type GetQueueStream = UserStream;

    async fn get_queue(&self, request: Request<GetQueueRequest>) -> Result<Response<Self::GetQueueStream>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn swipe(&self, swipe: Request<SwipeRequest>) -> Result<Response<SwipeResponse>, Status> {
        Err(Status::unimplemented("Not Implemented"))
    }

}