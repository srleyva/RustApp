use log::{debug, error, info};

use super::super::location::sharding::{GeoShard, MAX_SHARD};
use super::super::recommendation::User;
use super::indices::{GeoShardMappingIndex, UserIndex};
use elasticsearch::indices::IndicesCreateParts;

use serde_json::value::Value;

use elasticsearch::http::request::JsonBody;
use elasticsearch::{BulkParts, CreateParts, Elasticsearch, GetParts, SearchParts};

pub async fn build_geoshard_mapping_index(client: &Elasticsearch, shards: &[GeoShard]) {
    info!(
        "Building Geoshard Mapping Index: {}",
        GeoShardMappingIndex::name()
    );
    client.ping();
    client
        .indices()
        .create(IndicesCreateParts::Index(
            GeoShardMappingIndex::name().as_str(),
        ))
        .body(GeoShardMappingIndex::body())
        .send()
        .await
        .unwrap();

    let mut body: Vec<JsonBody<_>> = Vec::with_capacity(4);
    for shard in shards {
        body.push(json!({"index": {"_id": shard.name}}).into());
        body.push(serde_json::to_value(shard).unwrap().into());
    }

    let response = client
        .bulk(BulkParts::Index(GeoShardMappingIndex::name().as_str()))
        .body(body)
        .send()
        .await
        .unwrap();
    info!(
        "Sucess for mapping {}: {}",
        GeoShardMappingIndex::name(),
        response.status_code().is_success()
    );
}

pub async fn build_geosharded_indices(client: &Elasticsearch, shards: &[GeoShard]) {
    info!("Building shards from level {} index", 7);
    client.ping();

    for shard in shards {
        let user_index = UserIndex::from(shard);
        info!("Creating Index: {}", user_index.name());
        let response = client
            .indices()
            .create(IndicesCreateParts::Index(user_index.name().as_str()))
            .body(user_index.body())
            .send()
            .await
            .unwrap();
        info!(
            "Sucess for geoshard creation {}: {}",
            shard.name,
            response.status_code().is_success()
        );
    }
}

pub struct ElasticOperator {
    pub client: Elasticsearch,
}

impl ElasticOperator {
    pub fn new(client: Elasticsearch) -> Self {
        Self { client }
    }

    pub async fn get_user(&self, index: &str, uid: String) -> User {
        let resp = self
            .client
            .get(GetParts::IndexId(&index, uid.as_str()))
            .send()
            .await
            .unwrap();

        let json: Value = resp.json().await.unwrap();
        serde_json::from_value(json["_source"].clone()).unwrap()
    }

    pub async fn get_users(
        &self,
        indices: Vec<&str>,
        lat: f64,
        lon: f64,
        distance: u32,
        age_range: Vec<i32>,
        gender: u64,
    ) -> Vec<User> {
        let query = json!({
          "from": 0, "size": 1000,
          "query": {
            "bool": {
              "must": [
                {
                  "range": {
                    "age": {
                      "gte": age_range[0],
                      "lte": age_range[1]
                    }
                  }
                }
              ],
              "filter": [
                {
                  "term": {
                    "gender": gender
                  }
                },
                {
                  "geo_distance": {
                    "distance": format!("{}mi", distance), // TODO: Metric support
                    "location": { "lon": lon, "lat": lat }
                  }
                }
              ]
            }
          }
        });
        debug!("{}", query);
        let resp = self
            .client
            .search(SearchParts::Index(&indices.as_slice()))
            .body(query)
            .send()
            .await
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        debug!("{}", json);
        json["hits"]["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| serde_json::from_value(h["_source"].clone()).unwrap())
            .collect()
    }

    pub async fn write_user(&self, index: &str, user: User) {
        info!(
            "Writing User: {} {} to {}",
            user.first_name, user.last_name, index
        );
        info!("User: {}", serde_json::to_value(&user).unwrap().to_string());
        let resp = self
            .client
            .create(CreateParts::IndexId(index, &user.uid))
            .body(&user)
            .send()
            .await
            .unwrap()
            .exception()
            .await
            .unwrap();
        println!("{:?}", resp);
    }

    pub async fn write_users(&self, user_body: Vec<JsonBody<serde_json::Value>>) {
        info!("Bulk writing users: {}", user_body.len() / 2);
        let resp = self
            .client
            .bulk(BulkParts::None)
            .body(user_body)
            .send()
            .await
            .unwrap();
        match resp.exception().await {
            Ok(err) => {
                if let Some(err) = err {
                    error!("Error: {:?}", err)
                }
            }
            Err(err) => error!("{}", err),
        }
    }

    pub async fn load_shard_into_memory(&self) -> Vec<GeoShard> {
        info!(
            "Loading Shards from elastic: {}",
            GeoShardMappingIndex::name()
        );
        let resp = self
            .client
            .search(SearchParts::Index(&[GeoShardMappingIndex::name().as_str()]))
            .body(json!({ "size": MAX_SHARD }))
            .send()
            .await
            .unwrap()
            .error_for_status_code()
            .unwrap();
        let json: Value = resp.json().await.unwrap();
        debug!("Raw Shards {}", json["hits"]["hits"]);

        let shards: Vec<GeoShard> = json["hits"]["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| serde_json::from_value(h["_source"].clone()).unwrap())
            .collect();
        info!("Loaded {} shards into memory", shards.len());
        shards
    }
}
