use futures::Stream;
use std::pin::Pin;
use tonic::{
    Request, 
    Response, 
    Status
};
use super::elastic::ops::ElasticOperator;
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

use super::location::sharding::GeoShardSearcher;

pub struct MainRecommendactionService {
    elastic_operator: ElasticOperator,
    searcher: GeoShardSearcher
}

impl MainRecommendactionService {
    pub async fn new(elastic_operator: ElasticOperator) -> Self {
        let shards = elastic_operator.load_shard_into_memory().await;
        let searcher = GeoShardSearcher::from(shards);
        Self {
            elastic_operator,
            searcher
        }
    }
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