use log::{
    info,
    debug,
};


use super::super::location::sharding::{
    GeoShard,
    MAX_SHARD,
};
use super::indices::{
    GeoShardMappingIndex,
    UserIndex,
};
use super::super::recommendation::{
    User
};
use elasticsearch::indices::{
    IndicesCreateParts
};

use serde_json::{
    value::Value
};

use elasticsearch::{
    Elasticsearch,
    BulkParts,
    SearchParts
};
use elasticsearch::http::request::JsonBody;

pub async fn build_geoshard_mapping_index(client: &Elasticsearch, shards: &Vec<GeoShard>) {
    info!("Building Geoshard Mapping Index: {}", GeoShardMappingIndex::name());
    client.ping();
    client.indices()
        .create(IndicesCreateParts::Index(GeoShardMappingIndex::name().as_str()))
        .body(GeoShardMappingIndex::body())
        .send().await.unwrap();

    let mut body: Vec<JsonBody<_>> = Vec::with_capacity(4);
    for shard in shards {
        body.push(json!({"index": {"_id": shard.name}}).into());
        body.push(serde_json::to_value(shard).unwrap().into());
    }

    let response = client
    .bulk(BulkParts::Index(GeoShardMappingIndex::name().as_str()))
    .body(body)
    .send()
    .await.unwrap();
    info!("Sucess for mapping {}: {}", GeoShardMappingIndex::name(), response.status_code().is_success());
}

pub async fn build_geosharded_indices(client: &Elasticsearch, shards: &Vec<GeoShard>) {
    info!("Building shards from level {} index", 7);
    client.ping();

    for shard in shards {
        let user_index = UserIndex::from(shard);
        info!("Creating Index: {}", user_index.name());
        let response = client.indices()
            .create(IndicesCreateParts::Index(user_index.name().as_str()))
            .body(user_index.body())
            .send().await.unwrap();
        info!("Sucess for geoshard creation {}: {}", shard.name, response.status_code().is_success());
    }
}



pub struct ElasticOperator {
    client: Elasticsearch
}

impl ElasticOperator {
    pub fn new(client: Elasticsearch) -> Self {
        Self {
            client
        }
    }

    pub async fn load_shard_into_memory(&self) -> Vec<GeoShard> {
        info!("Loading Shards from elastic: {}", GeoShardMappingIndex::name());    
        let resp = self.client
            .search(SearchParts::Index(&[GeoShardMappingIndex::name().as_str()]))
            .body(json!({
                "size": MAX_SHARD
            }))
            .send().await.unwrap()
            .error_for_status_code().unwrap();
        
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