use elastic::prelude::*;
use elastic::client::{SyncClient};
use log::info;
use env_logger::init;
use rand::Rng;


use location::{Shards, LoadCount};
use s2::cellid::CellID;

mod location;

struct PopulationLoadCounter;

impl PopulationLoadCounter {
    fn  new() -> Self {
        Self
    }
}

impl LoadCount for PopulationLoadCounter {
    fn load_count(self, cell_id: CellID) -> i32 {
        let mut rng = rand::thread_rng();

        return match cell_id.face() {
            1..=2 => rng.gen_range(0, 5), // Oceans
            3..=4 => rng.gen_range(10, 500), // medium and small cities
            _ => rng.gen_range(1000, 2000) // Big Cities
        }
    }
}

fn main() {
    env_logger::init();
    let client = SyncClient::builder().sniff_nodes("http://localhost:9200").build().unwrap();
    client.ping().send().unwrap();
    build_index_from_geoshard(client);
}

fn build_index_from_geoshard(es_client: SyncClient) {
    init();
    info!("Building shards from level {} index", 7);
    let shard = Shards::new(7, PopulationLoadCounter::new());

    for geoshard in shard.shards {
        info!("Building index from geoshard: {}", geoshard.name);
        let shard_index = index(geoshard.name);
        let response = es_client.index(shard_index).create().send().unwrap();
        assert!(response.acknowledged());
    }
}