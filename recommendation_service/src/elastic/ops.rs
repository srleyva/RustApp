use log::info;


use super::super::location::sharding::GeoShard;
use super::indices::{
    GeoShardMappingIndex,
    UserIndex,
};

use elasticsearch::indices::{
    IndicesCreateParts
};

use elasticsearch::{
    Elasticsearch,
    BulkParts
};
use elasticsearch::http::request::JsonBody;

pub async fn build_geoshard_mapping_index(client: &Elasticsearch, shards: &Vec<GeoShard>) {
    info!("Building Geoshard Mapping Index: {}", GeoShardMappingIndex::name());
    client.ping();
    client.indices()
        .create(IndicesCreateParts::Index(GeoShardMappingIndex::name().as_str()))
        .body(GeoShardMappingIndex::body())
        .send().await.unwrap();
    
    for shard in shards {
        let mut body: Vec<JsonBody<_>> = Vec::with_capacity(4);
        body.push(json!({"index": {"_id": shard.name}}).into());
        body.push(serde_json::to_value(shard).unwrap().into());
        
        let response = client
            .bulk(BulkParts::Index(GeoShardMappingIndex::name().as_str()))
            .body(body)
            .send()
            .await.unwrap();
        info!("Sucess for mapping {}: {}",shard.name, response.status_code().is_success());
    }
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