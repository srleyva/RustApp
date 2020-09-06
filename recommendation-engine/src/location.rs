use std::collections::BTreeMap;
use std::thread;
use rand::Rng;


use s2::s1;
use s2::cellid::CellID;
use s2::latlng::LatLng;
use s2::point::Point;
use s2::cap::Cap;
use s2::region::RegionCoverer;

pub const EARTH_RADIUS: f64 = 6.37e6f64;

macro_rules! ll {
    ($lng:expr, $lat:expr) => {
        LatLng {
            lat: s1::Deg($lat).into(),
            lng: s1::Deg($lng).into(),
        }
    };
}

pub struct S2List {
    storage_level: u64,
}

impl S2List {
    pub fn new(storage_level: u64) -> Self {
        Self{
            storage_level,
        }
    }

    fn recursive_list(&self, cell_id: CellID, seen: &mut BTreeMap<CellID, i32>) {
        let neighbors = cell_id.vertex_neighbors(self.storage_level);
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

    pub fn into_list(self) -> BTreeMap<CellID, i32> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_s2_list() {
        let s2_list = S2List::new(8).into_list();    
        assert_eq!(s2_list.len(), 393216);
    }
}