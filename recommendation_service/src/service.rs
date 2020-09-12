use futures::{
    Stream,
    task::{
        Context,
        Poll
    }
};
use std::pin::Pin;
use tonic::{
    Request, 
    Response, 
    Status
};
use log::{
    info,
};
use super::elastic::ops::ElasticOperator;
use super::recommendation::{
    User,
    Gender,
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

    pub async fn new_users(&self, users: Vec<User>) {
        for user in users {
            let shard = self.searcher.get_shard_from_lng_lat(
                user.location.as_ref().unwrap().latitude,
                user.location.as_ref().unwrap().longitude
            );
            self.elastic_operator.write_user(&shard.name, user).await;
        }
        // Bulk write doesn't work due to SingleNodeConnectionPool
        // let body = self.searcher.build_es_request(users);
        // self.elastic_operator.write_users(body).await;
    }
}

pub struct UserStream {
    users: Vec<User>
}

impl UserStream {
    fn new() -> Self {
        // Request 1000 users from ES and stream, eventually cache in redis

        Self {
            users: vec![]
        }
    }
}

impl Stream for UserStream {
    type Item = Result<User, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        return match self.users.pop() {
                Some(user) => Poll::Ready(Some(Ok(user))),
                None => Poll::Ready(None),
        }
    }
}

#[tonic::async_trait]
impl RecommendationService for MainRecommendactionService {
    type GetQueueStream = UserStream;

    async fn get_queue(&self, request: Request<GetQueueRequest>) -> Result<Response<Self::GetQueueStream>, Status> {
        let request = request.into_inner();
        let user_shards = self.searcher.get_shards_from_radius(request.longitude, request.latitude, request.radius);
        
        let es_index = user_shards.into_iter().map(|x| &x.name);
        info!("User Query will hit {} shards", es_index.len());

        let user_stream = UserStream{
            users: vec![
                User{
                    first_name: "Stephen".to_string(),
                    last_name: "Leyva".to_string(),
                    uid: "some-uid".to_string(),
                    age: 32,
                    gender: Gender::Male as i32,
                    location: Some(Location {
                        latitude: 47.6062,
                        longitude: 122.3321
                    }),
                    my_swipes: Vec::new(),
                    potential_matches: Vec::new()
                },
                User{
                    first_name: "Stephen".to_string(),
                    last_name: "Leyva".to_string(),
                    uid: "some-uid".to_string(),
                    age: 32,
                    gender: Gender::Male as i32,
                    location: Some(Location {
                        latitude: 47.6062,
                        longitude: 122.3321
                    }),
                    my_swipes: Vec::new(),
                    potential_matches: Vec::new()
                },
                User{
                    first_name: "Stephen".to_string(),
                    last_name: "Leyva".to_string(),
                    uid: "some-uid".to_string(),
                    age: 32,
                    gender: Gender::Male as i32,
                    location: Some(Location {
                        latitude: 47.6062,
                        longitude: 122.3321
                    }),
                    my_swipes: Vec::new(),
                    potential_matches: Vec::new()
                }
                ,User{
                    first_name: "Stephen".to_string(),
                    last_name: "Leyva".to_string(),
                    uid: "some-uid".to_string(),
                    age: 32,
                    gender: Gender::Male as i32,
                    location: Some(Location {
                        latitude: 47.6062,
                        longitude: 122.3321
                    }),
                    my_swipes: Vec::new(),
                    potential_matches: Vec::new()
                }
                ,User{
                    first_name: "Stephen".to_string(),
                    last_name: "Leyva".to_string(),
                    uid: "some-uid".to_string(),
                    age: 32,
                    gender: Gender::Male as i32,
                    location: Some(Location {
                        latitude: 47.6062,
                        longitude: 122.3321
                    }),
                    my_swipes: Vec::new(),
                    potential_matches: Vec::new()
                }
            ]
        };

        Ok(Response::new(user_stream))
    }

    async fn swipe(&self, swipe: Request<SwipeRequest>) -> Result<Response<SwipeResponse>, Status> {
        Err(Status::unimplemented("Not Implemented"))
    }

}