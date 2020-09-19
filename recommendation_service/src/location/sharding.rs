use std::collections::BTreeMap;
use std::thread;

use super::super::recommendation::User;
use elasticsearch::http::request::JsonBody;

use log::{debug, info};
use s2::cap::Cap;
use s2::cellid::CellID;
use s2::latlng::LatLng;
use s2::point::Point;
use s2::region::RegionCoverer;
use s2::s1;

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

pub trait Scorer {
    fn score_list(&self, cell_list: CellList, users: &Vec<User>) -> CellList;
}

pub struct UserCountScorer;

impl Scorer for UserCountScorer {
    fn score_list(&self, mut cell_list: CellList, users: &Vec<User>) -> CellList {
        info!("Scoring Cells");
        for user in users {
            let location = user.location.as_ref().unwrap();
            let cell_id = cell_id_from_long_lat(
                location.longitude,
                location.latitude,
                cell_list.storage_level as u64,
            );
            let score = cell_list.cell_list.get_mut(&cell_id).unwrap();
            *score += 1;
        }
        cell_list
    }
}

pub struct CellList {
    storage_level: u64,
    cell_list: BTreeMap<CellID, i32>,
}

impl CellList {
    pub fn cell_list(storage_level: u64) -> Self {
        let starting_cell_id = CellID::from(ll!(0.000000, 0.000000));
        let mut cell_list = BTreeMap::new();
        // Thread to alter stack size
        // TODO Derive stack size based on imput
        let child = thread::Builder::new()
            .stack_size(50 * 1024 * 1024) // 45mb required for stack, 50 for the num gen
            .spawn(move || {
                Self::recursive_list(storage_level, starting_cell_id, &mut cell_list);
                cell_list
            })
            .unwrap();

        let cell_list = child.join().unwrap();
        Self {
            storage_level,
            cell_list,
        }
    }

    fn recursive_list(storage_level: u64, cell_id: CellID, seen: &mut BTreeMap<CellID, i32>) {
        let neighbors = cell_id.vertex_neighbors(storage_level);
        for neighbor in neighbors {
            match seen.get(&neighbor) {
                Some(_) => (),
                None => {
                    seen.insert(neighbor, 0);
                    Self::recursive_list(storage_level, neighbor, seen);
                }
            }
        }
    }
}

pub struct GeoshardBuilder<'a, ScoreStrategy> {
    pub storage_level: u64,
    users: &'a Vec<User>,
    score_strategy: ScoreStrategy,
}

// Constructors
impl<'a, ScoreStrategy> GeoshardBuilder<'a, ScoreStrategy>
where
    ScoreStrategy: Scorer,
{
    pub fn new(storage_level: u64, users: &'a Vec<User>, score_strategy: ScoreStrategy) -> Self {
        Self {
            storage_level,
            users,
            score_strategy,
        }
    }
}

impl<'a> GeoshardBuilder<'a, UserCountScorer> {
    pub fn user_count_scorer(storage_level: u64, users: &'a Vec<User>) -> Self {
        Self {
            storage_level,
            users,
            score_strategy: UserCountScorer,
        }
    }
}

impl<'a, ScoreStrategy> GeoshardBuilder<'a, ScoreStrategy>
where
    ScoreStrategy: Scorer,
{
    pub fn build(self) -> Vec<GeoShard> {
        let scored_cell_list = self
            .score_strategy
            .score_list(CellList::cell_list(self.storage_level), self.users);
        generate_shards(scored_cell_list)
    }
}

pub fn generate_shards(cell_list: CellList) -> Vec<GeoShard> {
    info!("Generating shards at level: {}", cell_list.storage_level);
    let cell_load = &cell_list.cell_list;
    let total: i32 = cell_load.iter().fold(0, |sum, i| sum + i.1);
    let max_size = total / MIN_SHARD;
    let min_size = total / MAX_SHARD;
    let mut best_shards: Vec<GeoShard> = vec![];
    let mut min_standard_deviation = f64::MAX;
    for container_size in min_size..=max_size {
        debug!(
            "Attempt {} out of {}",
            container_size - min_size,
            max_size - min_size
        );
        let first_cell = cell_load.iter().next().unwrap();
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
            if shard.start == None {
                shard.start = Some(cell_id.to_token());
            }
            if shard.cell_score + cell_score < container_size {
                shard.cell_score += cell_score;
                shard.cell_count += 1;
            } else {
                shard.end = Some(cell_id.to_token());
                geo_shards.push(shard);
                shard = GeoShard {
                    name: format!("geoshard_user_index_{}", geoshard_count),
                    storage_level: cell_id.level() as i64,
                    start: None,
                    end: None,
                    cell_count: 0,
                    cell_score: *cell_score,
                };
                geoshard_count += 1;
            }
        }
        if shard.cell_count != 0 {
            let last = cell_load.iter().last().unwrap();
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

pub fn cell_id_from_long_lat(long: f64, lat: f64, storage_level: u64) -> CellID {
    let long_lat = ll!(long, lat);
    let cell_id = CellID::from(long_lat).parent(storage_level);
    info!("Level: {}, Cell: {}", cell_id.level(), cell_id.to_token());
    cell_id
}

pub fn cell_ids_from_radius(long: f64, lat: f64, storage_level: u64, radius: u32) -> Vec<CellID> {
    let lon_lat = ll!(long, lat);

    let center_point = Point::from(lon_lat);

    let center_angle = s1::Deg(radius as f64 / EARTH_RADIUS).into();

    let cap = Cap::from_center_angle(&center_point, &center_angle);

    let region_cover = RegionCoverer {
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
    pub fn get_shard_from_cell_id(&self, cell_id: CellID) -> &GeoShard {
        for geoshard in &self.shards {
            // Check if cell_id in shard
            let cell_id_start = CellID::from_token(geoshard.start.as_ref().unwrap().as_str());
            let cell_id_end = CellID::from_token(geoshard.end.as_ref().unwrap().as_str());
            debug!(
                "Range: {}-{} Value: {}",
                cell_id_start, cell_id_end, cell_id
            );
            if cell_id >= cell_id_start && cell_id <= cell_id_end {
                return &geoshard;
            }
        }
        self.shards.last().unwrap()
    }

    pub fn get_shard_from_lng_lat(&self, lng: f64, lat: f64) -> &GeoShard {
        let cell_id = cell_id_from_long_lat(lng, lat, self.storage_level as u64);
        debug!("{} {} => cell: {}", lat, lng, cell_id.to_token());
        self.get_shard_from_cell_id(cell_id)
    }

    pub fn get_shards_from_radius(&self, lng: f64, lat: f64, radius: u32) -> Vec<&GeoShard> {
        let mut geoshards = vec![];
        let cell_ids = cell_ids_from_radius(lng, lat, self.storage_level as u64, radius);
        for cell_id in cell_ids {
            geoshards.push(self.get_shard_from_cell_id(cell_id));
        }
        geoshards
    }

    pub fn build_es_request(&self, users: Vec<User>) -> Vec<JsonBody<serde_json::Value>> {
        let mut body: Vec<JsonBody<_>> = Vec::with_capacity(4);

        for user in users {
            let index = self.get_shard_from_lng_lat(
                user.location.as_ref().unwrap().longitude,
                user.location.as_ref().unwrap().latitude,
            );
            debug!(
                "Creating User {} in shard {}",
                format!("{} {}", user.first_name, user.last_name),
                index.name
            );
            body.push(json!({"index": {"_index": index.name, "_id": user.uid }}).into());
            body.push(serde_json::to_value(&user).unwrap().into());
        }
        body
    }
}

impl From<Vec<GeoShard>> for GeoShardSearcher {
    fn from(shards: Vec<GeoShard>) -> Self {
        let storage_level = shards.first().unwrap().storage_level;
        Self {
            storage_level,
            shards,
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
    cell_score: i32,
}

pub fn standard_deviation_between_shards(shards: &[GeoShard]) -> f64 {
    let mean: f64 =
        shards.iter().fold(0.0, |sum, x| sum + x.cell_score as f64) / shards.len() as f64;

    let varience: f64 = shards
        .iter()
        .map(|x| (x.cell_score as f64 - mean) * (x.cell_score as f64 - mean))
        .sum::<f64>()
        / shards.len() as f64;

    varience.sqrt()
}

#[cfg(test)]
mod test {
    use super::*;

    use rand::Rng;

    use s2::cellid::CellID;
    use s2::latlng::LatLng;
    use s2::s1;

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
        let cell_list = CellList::cell_list(8).cell_list;
        assert_eq!(cell_list.len(), 393216);
    }

    #[test]
    fn test_shard_search() {
        let geoshard = GeoshardBuilder::user_count_scorer(4, &vec![]).build();
        let geoshards = GeoShardSearcher::from(geoshard);

        let geoshard = geoshards.get_shard_from_lng_lat(34.181061, -103.345177);

        let cell_id = cell_id_from_long_lat(34.181061, -103.345177, 4);
        let cell_id_start = CellID::from_token(geoshard.start.as_ref().unwrap().as_str());
        let cell_id_end = CellID::from_token(geoshard.end.as_ref().unwrap().as_str());

        let range = cell_id_start..=cell_id_end;
        println!("Geoshard Range: {}-{}", cell_id_start, cell_id_end);
        println!("Geoshard cell: {}", cell_id);
        assert!(range.contains(&cell_id));
    }

    #[test]
    fn test_shard_radius_search() {
        let geoshard = GeoshardBuilder::user_count_scorer(4, &vec![]).build();
        let geoshards = GeoShardSearcher::from(geoshard);
        let geoshards = geoshards.get_shards_from_radius(34.181061, -103.345177, 200);
        assert_eq!(geoshards.len(), 1);
    }

    #[test]
    fn test_generate_shards() {
        let cell_list = CellList {
            storage_level: 4,
            cell_list: generate_random_cell_load(),
        };

        let shards = generate_shards(cell_list);

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
