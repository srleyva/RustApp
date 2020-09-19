use super::elastic::ops::ElasticOperator;
use super::recommendation::{
    recommendation_service_server::RecommendationService, GetQueueRequest, SwipeRequest,
    SwipeResponse, User,
};
use futures::{
    task::{Context, Poll},
    Stream,
};
use log::info;
use std::pin::Pin;
use tonic::{Request, Response, Status};

use super::location::sharding::GeoShardSearcher;

pub struct MainRecommendactionService {
    elastic_operator: ElasticOperator,
    searcher: GeoShardSearcher,
}

impl MainRecommendactionService {
    pub async fn new(elastic_operator: ElasticOperator) -> Self {
        let shards = elastic_operator.load_shard_into_memory().await;
        let searcher = GeoShardSearcher::from(shards);
        Self {
            elastic_operator,
            searcher,
        }
    }

    pub async fn new_users(&self, users: Vec<User>) {
        for user in users {
            let shard = self.searcher.get_shard_from_lng_lat(
                user.location.as_ref().unwrap().longitude,
                user.location.as_ref().unwrap().latitude,
            );
            self.elastic_operator.write_user(&shard.name, user).await;
        }
        // Bulk write doesn't work due to SingleNodeConnectionPool
        // let body = self.searcher.build_es_request(users);
        // self.elastic_operator.write_users(body).await;
    }
}

pub struct UserStream {
    users: Vec<User>,
}

impl Stream for UserStream {
    type Item = Result<User, Status>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.users.pop() {
            Some(user) => Poll::Ready(Some(Ok(user))),
            None => Poll::Ready(None),
        }
    }
}

#[tonic::async_trait]
impl RecommendationService for MainRecommendactionService {
    type GetQueueStream = UserStream;

    async fn get_queue(
        &self,
        request: Request<GetQueueRequest>,
    ) -> Result<Response<Self::GetQueueStream>, Status> {
        let request = request.into_inner();
        let user_shards = self.searcher.get_shards_from_radius(
            request.longitude,
            request.latitude,
            request.radius,
        );
        let es_index: Vec<&str> = user_shards.into_iter().map(|x| x.name.as_str()).collect();
        info!("User Query will hit {:?} shards", es_index);
        let users = self
            .elastic_operator
            .get_users(
                es_index,
                request.latitude,
                request.longitude,
                request.radius,
                request.age_range,
                request.gender as u64,
            )
            .await;

        let user_stream = UserStream { users };

        Ok(Response::new(user_stream))
    }

    async fn swipe(
        &self,
        _swipe: Request<SwipeRequest>,
    ) -> Result<Response<SwipeResponse>, Status> {
        Err(Status::unimplemented("Not Implemented"))
    }
}
