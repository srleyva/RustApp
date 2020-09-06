use std::collections::BTreeMap;
use s2::cellid::CellID;

use crate::location::{cell_id_from_long_lat, S2List, cell_ids_from_radius};

const MIN_SHARD: i32 = 40;
const MAX_SHARD: i32 = 100;

pub struct Shards {
    storage_level: u64,
    shards: Vec<Shard>,
}

impl Shards {

    pub fn new(storage_level: u64) -> Self {
        let cells = S2List::new(storage_level).into_list();
        let shards = generate_shards(&cells);

        Self{
            storage_level,
            shards
        }
    }
    
    pub fn get_shard_from_cell_id(&self, cell_id: CellID) -> Option<&Shard> {
        for geoshard in &self.shards {
            // Check if cell_id in shard
            if cell_id >= geoshard.ranges.start.unwrap() && cell_id < geoshard.ranges.end.unwrap() {
                return Some(&geoshard);
            }
        }
        None
    }

    pub fn get_shard_from_lng_lat(&self, lng: f64, lat: f64) -> Option<&Shard> {
        let cell_id = cell_id_from_long_lat(lng, lat, self.storage_level);        
        self.get_shard_from_cell_id(cell_id)
    }

    pub fn get_shards_from_radius(&self, lng: f64, lat: f64, radius: u32) -> Vec<&Shard> {
        let mut geoshards = vec![];
        let cell_ids = cell_ids_from_radius(lng, lat, self.storage_level, radius);
        for cell_id in cell_ids {
            geoshards.push(self.get_shard_from_cell_id(cell_id).unwrap())
        }

        geoshards
    }
}

#[derive(Debug)]
pub struct Ranges {
    start: Option<CellID>,
    end: Option<CellID>
}

#[derive(Debug)]
pub struct Shard {
    ranges: Ranges,
    cell_count: i32,
    cell_score: i32
}

pub fn generate_shards(cell_load: &BTreeMap<CellID, i32>) -> Vec<Shard> {
    let total: i32 = cell_load.into_iter()
        .fold(0, |sum, i| sum + i.1);

    let max_size = total / MIN_SHARD;
    let min_size = total / MAX_SHARD;

    let mut best_shards: Vec<Shard> = vec![];
    let mut min_standard_deviation = f64::MAX;

    for container_size in min_size..=max_size {
        let mut shard = Shard {
            ranges: Ranges {
                start: Some(*cell_load.into_iter().next().unwrap().0),
                end: None,
            },
            cell_count: 0,
            cell_score: 0,
        };
        let mut geo_shards = Vec::new();
        for (cell_id, cell_score) in cell_load {
            if shard.cell_score + cell_score < container_size {
                shard.cell_score += cell_score;
                shard.cell_count += 1;
            } else {
                shard.ranges.end = Some(*cell_id);
                geo_shards.push(shard);
                shard = Shard {
                    ranges: Ranges {
                        start: Some(*cell_id),
                        end: None,
                    },
                    cell_count: 0,
                    cell_score: *cell_score,
                };            
            }
        }

        if shard.cell_count != 0 {
            let last = cell_load.into_iter().last().unwrap();
            shard.ranges.start = Some(*last.0);
            shard.ranges.end = Some(*last.0);
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

pub fn standard_deviation_between_shards(shards: &Vec<Shard>) -> f64 {
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
            Shard {
                ranges: Ranges {
                    start: None,
                    end: None,
                },
                cell_count: 0,
                cell_score: $cell_score,
            };

        };
    }

    #[test]
    fn test_shard_search() {
        let shards = Shards::new(4);
        let geoshard = shards.get_shard_from_lng_lat(34.181061, -103.345177).unwrap();

        let cell_id = cell_id_from_long_lat(34.181061, -103.345177, 4);

        let range = geoshard.ranges.start.unwrap()..geoshard.ranges.end.unwrap();
        assert!(range.contains(&cell_id));
    }

    #[test]
    fn test_shard_radius_search() {
        let shards = Shards::new(4);
        let geoshards = shards.get_shards_from_radius(34.181061, -103.345177, 200);
        assert_eq!(geoshards.len(), 1);
    }

    #[test]
    fn test_generate_shards() {
        let mock_cell_load = generate_random_cell_load();
        let shards = generate_shards(&mock_cell_load);

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