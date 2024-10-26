use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Read},
    time::Instant,
};
use rayon::prelude::*;

#[derive(Default, Debug, Clone)]
struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
}

impl Stats {
    fn merge(&mut self, other: &Stats) {
        if other.count > 0 {
            if self.count == 0 {
                self.min = other.min;
                self.max = other.max;
            } else {
                if other.min < self.min {
                    self.min = other.min;
                }
                if other.max > self.max {
                    self.max = other.max;
                }
            }
            self.sum += other.sum;
            self.count += other.count;
        }
    }
}

fn main() {
    let start = Instant::now();
    let file = File::open("../measurements.txt").expect("Error while trying to open file");
    let mut reader = BufReader::new(file);

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).expect("Failed to read file");

    let lines: Vec<&str> = buffer.lines().collect();
    let chunk_size = lines.len() / rayon::current_num_threads().max(1);

    let results: Vec<HashMap<String, Stats>> = lines
        .par_chunks(chunk_size)
        .map(|chunk| {
            let mut local_data = HashMap::new();
            for line in chunk {
                if let Some((city, temp)) = line.split_once(';') {
                    let city = city.trim().to_string();
                    let temp = temp.trim().parse::<f64>().expect("Failed to parse temp");

                    let stats = local_data.entry(city).or_insert_with(Stats::default);

                    if stats.count == 0 {
                        stats.min = temp;
                        stats.max = temp;
                    } else {
                        if temp < stats.min {
                            stats.min = temp;
                        }
                        if temp > stats.max {
                            stats.max = temp;
                        }
                    }

                    stats.sum += temp;
                    stats.count += 1;
                }
            }
            local_data
        })
        .collect();

    let mut city_data = HashMap::new();
    for local_data in results {
        for (city, local_stats) in local_data {
            city_data
                .entry(city)
                .or_insert_with(Stats::default)
                .merge(&local_stats);
        }
    }

    let mut sorted_city_data: Vec<_> = city_data.into_iter().collect();
    sorted_city_data.par_sort_unstable_by_key(|(city, _)| city.clone());

    print!("{{");
    let mut first = true;
    for (city, stats) in &sorted_city_data {
        if stats.count > 0 {
            let average = stats.sum / stats.count as f64;

            if !first {
                print!(", ");
            }
            first = false;

            print!(
                "{}={:.1}/{:.1}/{:.1}",
                city,
                stats.min,
                average,
                stats.max
            );
        }
    }
    println!("}}");

    let duration = start.elapsed();
    println!("Time taken: {:.3} seconds", duration.as_secs_f64());
}