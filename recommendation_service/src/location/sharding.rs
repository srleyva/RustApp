use std::collections::BTreeMap;
use std::thread;
use rand::Rng;

use s2::s1;
use s2::cellid::CellID;
use s2::latlng::LatLng;
use s2::point::Point;
use s2::cap::Cap;
use s2::region::RegionCoverer;
use log::{info};

pub const EARTH_RADIUS: f64 = 6.37e6f64;
pub const MIN_SHARD: i32 = 40;
pub const MAX_SHARD: i32 = 100;

macro_rules! ll {
    ($lng:expr, $lat:expr) => {
        LatLng {
            lat: s1::Deg($lat).into(),
            lng: s1::Deg($lng).into(),
        }
    };
}

pub struct GeoshardBuilder {
    pub storage_level: i64,
}

impl GeoshardBuilder {
    pub fn new(storage_level: i64) -> Self {
        info!("Generating shards at level: {}", storage_level);
        Self{
            storage_level,
        }
    }

    pub fn generate_shards(cell_load: BTreeMap<CellID, i32>) -> Vec<GeoShard> {
        let cell_load = &cell_load;
        let total: i32 = cell_load.into_iter()
            .fold(0, |sum, i| sum + i.1);
    
        let max_size = total / MIN_SHARD;
        let min_size = total / MAX_SHARD;
    
        let mut best_shards: Vec<GeoShard> = vec![];
        let mut min_standard_deviation = f64::MAX;
    
        for container_size in min_size..=max_size {
            info!("Attempt {} out of {}", container_size - min_size, max_size - min_size);
            let first_cell = cell_load.into_iter().next().unwrap();
            let mut shard = GeoShard {
                name: "geoshard_user_index_0".to_owned(),
                storage_level: first_cell.0.level() as i64,
                start: Some(first_cell.0.to_token()),
                end: None,
                cell_count: 0,
                cell_score: 0,
            };
            let mut geo_shards = Vec::new();
            let mut geoshard_count = 1;
            for (cell_id, cell_score) in cell_load {
                if shard.cell_score + cell_score < container_size {
                    shard.cell_score += cell_score;
                    shard.cell_count += 1;
                } else {
                    shard.end = Some(cell_id.to_token());
                    geo_shards.push(shard);
                    shard = GeoShard {
                        name: format!("geoshard_user_index_{}", geoshard_count),
                        storage_level: cell_id.level() as i64,
                        start: Some(cell_id.to_token()),
                        end: None,
                        cell_count: 0,
                        cell_score: *cell_score,
                    };
                    geoshard_count += 1;
                }
            }
    
            if shard.cell_count != 0 {
                let last = cell_load.into_iter().last().unwrap();
                shard.start = Some(last.0.to_token());
                shard.end = Some(last.0.to_token());
                shard.cell_count += 1;
                geo_shards.push(shard);
            }
    
            let standard_dev = standard_deviation_between_shards(&geo_shards);
            if standard_dev < min_standard_deviation {
                min_standard_deviation = standard_dev;
                best_shards = geo_shards;
            }
        }
        best_shards
    }

    fn recursive_list(&self, cell_id: CellID, seen: &mut BTreeMap<CellID, i32>) {
        let neighbors = cell_id.vertex_neighbors(self.storage_level as u64);
        for neighbor in neighbors {
            match seen.get(&neighbor) {
                Some(_) => (),
                None => {
                    let mut rng = rand::thread_rng();
                    seen.insert(neighbor, rng.gen_range(0, 2000)); // gen random range to represent load
                    self.recursive_list(neighbor, seen);
                }
            }
        }
    }

    pub fn into_cell_list(self) -> BTreeMap<CellID, i32> {
        let starting_cell_id = CellID::from(ll!(0.000000,0.000000));
        let mut seen = BTreeMap::new();

        let child = thread::Builder::new()
        .stack_size(50 * 1024 * 1024) // 45mb required for stack, 50 for the num gen
        .spawn(
            move || {
                self.recursive_list(starting_cell_id, &mut seen);
                seen
            }
        )
        .unwrap();

        child.join().unwrap()
    }

    pub fn into_geosharded_list(self) -> Vec<GeoShard> {
        GeoshardBuilder::generate_shards(self.into_cell_list())
    }
}

pub fn cell_id_from_long_lat(long: f64, lat: f64, storage_level: u64) -> CellID {
    let long_lat = ll!(long, lat);
    CellID::from(long_lat).parent(storage_level)
}

pub fn cell_ids_from_radius(long: f64, lat: f64, storage_level: u64, radius: u32) -> Vec<CellID> {
    let lon_lat = ll!(long, lat);

    let center_point = Point::from(lon_lat);

    let center_angle = s1::Deg(radius as f64 / EARTH_RADIUS).into();

    let cap = Cap::from_center_angle(&center_point, &center_angle);

    let region_cover = RegionCoverer{
        max_level: storage_level as u8,
        min_level: storage_level as u8,
        level_mod: 0,
        max_cells: 0,
    };
    
    region_cover.covering(&cap).0
}

pub struct GeoShardSearcher {
    storage_level: i64,
    pub shards: Vec<GeoShard>,
}

impl GeoShardSearcher {    
    pub fn get_shard_from_cell_id(&self, cell_id: CellID) -> Option<&GeoShard> {
        for geoshard in &self.shards {
            // Check if cell_id in shard
            let cell_id_start = CellID::from_token(geoshard.start.as_ref().unwrap().as_str());
            let cell_id_end = CellID::from_token(geoshard.end.as_ref().unwrap().as_str());

            if cell_id >= cell_id_start && cell_id < cell_id_end {
                return Some(&geoshard);
            }
        }
        None
    }

    pub fn get_shard_from_lng_lat(&self, lng: f64, lat: f64) -> Option<&GeoShard> {
        let cell_id = cell_id_from_long_lat(lng, lat, self.storage_level as u64);        
        self.get_shard_from_cell_id(cell_id)
    }

    pub fn get_shards_from_radius(&self, lng: f64, lat: f64, radius: u32) -> Vec<&GeoShard> {
        let mut geoshards = vec![];
        let cell_ids = cell_ids_from_radius(lng, lat, self.storage_level as u64, radius);
        for cell_id in cell_ids {
            geoshards.push(self.get_shard_from_cell_id(cell_id).unwrap())
        }

        geoshards
    }
}

impl From<GeoshardBuilder> for GeoShardSearcher {
    fn from(geoshard_builder: GeoshardBuilder) -> Self {
        Self {
            storage_level: geoshard_builder.storage_level,
            shards: geoshard_builder.into_geosharded_list(),
        }
    }
}

impl From<Vec<GeoShard>> for GeoShardSearcher {
    fn from(shards: Vec<GeoShard>) -> Self {
        let storage_level = shards.first().unwrap().storage_level;
        Self {
            storage_level,
            shards
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoShard {
    pub name: String,
    pub storage_level: i64,
    start: Option<String>,
    end: Option<String>,
    cell_count: i32,
    cell_score: i32
}

pub fn standard_deviation_between_shards(shards: &Vec<GeoShard>) -> f64 {
    let mean: f64 = shards.into_iter().fold(0.0, |sum, x| sum + x.cell_score as f64) / shards.len() as f64;

    let varience: f64 = shards.into_iter()
        .map(|x| (x.cell_score as f64 - mean) * (x.cell_score as f64 - mean)).sum::<f64>() / shards.len() as f64;

    varience.sqrt()
}


#[cfg(test)]
mod test {
    use super::*;

    use rand::Rng;

    use s2::s1;
    use s2::cellid::CellID;
    use s2::latlng::LatLng;

    macro_rules! ll {
        ($lat:expr, $lng:expr) => {
            LatLng {
                lat: s1::Deg($lat).into(),
                lng: s1::Deg($lng).into(),
            }
        };
    }

    macro_rules! shard {
        ($cell_score:expr) => {
            GeoShard {
                name: "fake".to_owned(),
                storage_level: 0,
                start: None,
                end: None,
                cell_count: 0,
                cell_score: $cell_score,
            };

        };
    }

    #[test]
    fn test_geoshard_cell_list() {
        let cell_list = GeoshardBuilder::new(8).into_cell_list();    
        assert_eq!(cell_list.len(), 393216);
    }

    #[test]
    fn test_shard_search() {
        let geoshards = GeoShardSearcher::from(GeoshardBuilder::new(4));

        let geoshard = geoshards.get_shard_from_lng_lat(34.181061, -103.345177).unwrap();

        let cell_id = cell_id_from_long_lat(34.181061, -103.345177, 4);
        let cell_id_start = CellID::from_token(geoshard.start.as_ref().unwrap().as_str());
        let cell_id_end = CellID::from_token(geoshard.end.as_ref().unwrap().as_str());


        let range = cell_id_start..cell_id_end;
        println!("Geoshard Range: {}-{}", cell_id_start, cell_id_end);
        println!("Geoshard cell: {}", cell_id);
        assert!(range.contains(&cell_id));
    }

    #[test]
    fn test_shard_radius_search() {
        let geoshards = GeoShardSearcher::from(GeoshardBuilder::new(4));
        let geoshards = geoshards.get_shards_from_radius(34.181061, -103.345177, 200);
        assert_eq!(geoshards.len(), 1);
    }

    #[test]
    fn test_generate_shards() {
        let mock_cell_load = generate_random_cell_load();
        let shards = GeoshardBuilder::generate_shards(mock_cell_load);

        if (shards.len() as i32) > MAX_SHARD || (shards.len() as i32) < MIN_SHARD {
            panic!("Shard len out of range: {}", shards.len());
        }
    }

    fn generate_random_cell_load() -> BTreeMap<CellID, i32> {
        let mut mock_values = BTreeMap::new();
        let mut rng = rand::thread_rng();

        // Ocean
        for _ in 0..=1000 {
            let rand_lat = rng.gen_range(0.000000, 2000.000000);
            let rand_long = rng.gen_range(0.000000, 2000.000000);

            let cell_id = CellID::from(ll!(rand_lat, rand_long));
            let rand_load_count = rng.gen_range(0, 5);
            mock_values.insert(cell_id, rand_load_count);
        }

        // Small Cities
        for _ in 0..=100 {
            let rand_lat = rng.gen_range(0.000000, 2000.000000);
            let rand_long = rng.gen_range(0.000000, 2000.000000);

            let cell_id = CellID::from(ll!(rand_lat, rand_long));
            let rand_load_count = rng.gen_range(10, 100);
            mock_values.insert(cell_id, rand_load_count);
        }

        // Medium Cities
        for _ in 0..=50 {
            let rand_lat = rng.gen_range(0.000000, 2000.000000);
            let rand_long = rng.gen_range(0.000000, 2000.000000);

            let cell_id = CellID::from(ll!(rand_lat, rand_long));
            let rand_load_count = rng.gen_range(100, 500);
            mock_values.insert(cell_id, rand_load_count);

        }

        // Big Cities
        for _ in 0..=10 {
            let rand_lat = rng.gen_range(0.000000, 2000.000000);
            let rand_long = rng.gen_range(0.000000, 2000.000000);

            let cell_id = CellID::from(ll!(rand_lat, rand_long));
            let rand_load_count = rng.gen_range(1000, 2000);
            mock_values.insert(cell_id, rand_load_count);
        }
        mock_values
    }

    #[test]
    fn test_standard_deviation() {
        let shards = vec![
            shard!(9),
            shard!(2),
            shard!(5),
            shard!(4),
            shard!(12),
            shard!(7),
            shard!(8),
            shard!(11),
            shard!(9),
            shard!(3),
            shard!(7),
            shard!(4),
            shard!(12),
            shard!(5),
            shard!(4),
            shard!(10),
            shard!(9),
            shard!(6),
            shard!(9),
            shard!(4),
        ];

        let standard_dev = standard_deviation_between_shards(&shards);
        assert_eq!(standard_dev, 2.9832867780352594_f64)
    }
}