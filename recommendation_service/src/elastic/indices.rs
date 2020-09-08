
use super::super::location::sharding::GeoShard;

pub struct GeoShardMappingIndex;

impl GeoShardMappingIndex {
    pub fn name() -> String {
        String::from("geoshard_mapping_index")
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
    pub fn name(self) -> String {
        format!("user_index_{}", self.geoshard.name)
    }
}