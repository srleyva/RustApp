use std::collections::BTreeMap;

use s2::s1;
use s2::region::RegionCoverer;
use s2::cap::Cap;
use s2::cellid::CellID;
use s2::point::Point;
use s2::latlng::LatLng;

pub const EARTH_RADIUS: f64 = 6.37e6f64;

macro_rules! ll {
    ($lat:expr, $lng:expr) => {
        LatLng {
            lat: s1::Deg($lat).into(),
            lng: s1::Deg($lng).into(),
        }
    };
}

struct Index {
    storage_level: u64,
    btree: BTreeMap<CellID, Vec<String>>, // Cell ID and user ids
}

impl Index {
    fn new(storage_level: u64) -> Self {
        Self{
            storage_level,
            btree: BTreeMap::new(),
        }
    }

    fn add_user(&mut self, uid: String, lon: f64, lat: f64) -> Result<(), String> {
        let latlng = ll!(lat, lon);
        let cell_id = CellID::from(latlng);
        let cell_id_storage_level = cell_id.parent(self.storage_level);

        match self.btree.get_mut(&cell_id_storage_level) {
            Some(ul) => {
                ul.push(uid);
            },
            None => {
                let ul = vec![uid];
                self.btree.insert(cell_id_storage_level, ul);
            },
        };

        Ok(())
    }

    fn search(&mut self, lon: f64, lat: f64, radius: u32) -> Result<Vec<String>, String> {
        let latlng = ll!(lat, lon);

        let center_point = Point::from(latlng);
        let center_angle = s1::Deg(radius as f64 / EARTH_RADIUS).into();

        let cap = Cap::from_center_angle(&center_point, &center_angle); 

        let region_cover = RegionCoverer{
            max_level: self.storage_level as u8,
            min_level: self.storage_level as u8,
            level_mod: 0,
            max_cells: 0,
        };

        let cell_ids = region_cover.covering(&cap).0;

        Ok(
            cell_ids.into_iter()
                .fold(vec![], |mut acc, x| match self.btree.get_mut(&x) {
                    Some(item) => {
                        acc.append(item);
                        acc
                    },
                    None => acc,
                }
            
            )
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search() {
        let mut index = Index::new(13);
        index.add_user("Test".to_owned(), 19.08563, 85.83803).unwrap();
        index.add_user("Test-1".to_owned(), 16.08563, 85.83803).unwrap();
        index.add_user("Test-2".to_owned(), 20.08563, 85.83803).unwrap();
        index.add_user("Test-3".to_owned(), 26.08563, 85.83803).unwrap();
        index.add_user("Test-3".to_owned(), 26.08564, 85.83803).unwrap();
        index.add_user("Test-3".to_owned(), 26.08565, 85.83803).unwrap();
        index.add_user("Test-4".to_owned(), 46.08563, 85.83803).unwrap();
        index.add_user("Test-5".to_owned(), 56.08563, 85.83803).unwrap();

        let found = match index.search(26.08563, 85.83803, 2000) {
            Ok(result) => result,
            Err(err) => panic!("{}", err)
        };

        assert_eq!(found.len(), 3);
    }
}