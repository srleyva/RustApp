use std::collections::HashMap;
use std::thread;
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

pub struct S2List {
    storage_level: u64,
}

impl S2List {
    pub fn new(storage_level: u64) -> Self {
        Self{
            storage_level,
        }
    }

    fn recursive_list(&self, cell_id: CellID, seen: &mut HashMap<CellID, i32>) {
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

    pub fn into_list(self) -> HashMap<CellID, i32> {
        let starting_cell_id = CellID::from(ll!(0.000000,0.000000));
        let mut seen = HashMap::new();

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_s2_list() {
        let s2_list = S2List::new(8).into_list();    
        assert_eq!(s2_list.len(), 393216);
    }
}