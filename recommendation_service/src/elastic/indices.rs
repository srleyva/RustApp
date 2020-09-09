
use super::super::location::sharding::GeoShard;
use serde_json::Value;

pub struct GeoShardMappingIndex;

impl GeoShardMappingIndex {
    pub fn name() -> String {
        String::from("geoshard_mapping_index")
    }

    pub fn body() -> Value {
        json!({
            "mappings" : {
                "properties" : {
                    "name" : { "type" : "text" },
                    "storage_level" : { "type" : "long" },
                    "start" : { "type" : "text" },
                    "end": {"type" : "text" },
                    "cell_count": { "type": "integer" },
                    "cell_score": { "type": "integer" },
                }
            }
        })
    }
}

pub struct UserIndex<'a> {
    geoshard: &'a GeoShard
}

impl<'a> From<&'a GeoShard> for UserIndex<'a> {
    fn from(geoshard: &'a GeoShard) -> Self {
        Self {
            geoshard
        }
    }
}

impl<'a> UserIndex<'a> {
    pub fn name(&self) -> &String {
        &self.geoshard.name
    }

    pub fn body(&self) -> Value {
        json!({
            "mappings" : {
                "properties" : {
                    "uid": {"type": "text"},
                    "first_name" : { "type" : "text" },
                    "last_name" : { "type" : "text" },
                    "age" : { "type" : "integer" },
                    "location": {"type" : "geo_point" },
                }
            }
        })
    }
}