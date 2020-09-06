
const MIN_SHARD: i32 = 40;
const MAX_SHARD: i32 = 100;

pub fn standard_deviation(shards: &Vec<i32>) -> f64 {
    let mean: f64 = shards.into_iter().sum::<i32>() as f64 / shards.len() as f64;

    let varience: f64 = shards.into_iter()
        .map(|x| (*x as f64 - mean) * (*x as f64 - mean)).sum::<f64>() / shards.len() as f64;
    
    varience.sqrt()
}

pub fn generate_shards(cell_load: &Vec<i32>) -> Vec<i32> {
    let total: i32 = cell_load.into_iter().sum::<i32>();

    let max_size = total / MIN_SHARD;
    let min_size = total / MAX_SHARD;

    let mut best_shards: Vec<i32> = vec![];
    let mut min_standard_deviation = f64::MAX;

    for i in min_size..max_size {
        let mut current_sum = 0;
        let mut geo_shards = vec![];
        for cell_score in cell_load {
            if current_sum + cell_score < i {
                current_sum += cell_score;
            } else {
                geo_shards.push(current_sum);
                current_sum += cell_score;
            }
        }

        if current_sum != 0 {
            geo_shards.push(current_sum);
        }

        let standard_dev = standard_deviation(&geo_shards);
        if standard_dev < min_standard_deviation {
            min_standard_deviation = standard_dev;
            best_shards = geo_shards;
        }
    }
    best_shards
}

#[cfg(test)]
mod test {
    use super::*;

    use rand::Rng;

    #[test]
    fn test_generate_shards() {
        let mock_cell_load = generate_random_cell_load(100);
        let shards = generate_shards(&mock_cell_load);

        if (shards.len() as i32) > MAX_SHARD || (shards.len() as i32) < MIN_SHARD {
            panic!("Shard len out of range: {}", shards.len());
        }
    }

    fn generate_random_cell_load(count: i32) -> Vec<i32> {
        let mut rng = rand::thread_rng();
        (0..count).map(|_| rng.gen_range(0, 2000)).collect()
    }

    #[test]
    fn test_standard_deviation() {
        let nums = vec![ 9, 2, 5, 4, 12, 7, 8, 11, 9, 3, 7, 4, 12, 5, 4, 10, 9, 6, 9, 4 ];
        let standard_dev = standard_deviation(&nums);
        assert_eq!(standard_dev, 2.9832867780352594_f64)
    }
}