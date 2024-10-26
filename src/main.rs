use std::{collections::{BTreeMap, HashMap}, fs::File, io::{self, BufRead}, time::Instant};

struct Stats {
    avg: i32,
    min: i32,
    max: i32,
}

fn main() {
    let start = Instant::now();
    let file = File::open("../measurements.txt").expect("Error whilre trying to open file");
    let reader = io::BufReader::new(file);

    #[derive(Default)]
    struct Stats {
        min: f64,
        max: f64,
        sum: f64,
        count: usize,
    }
  

    let mut city_data: HashMap<String, Stats> = HashMap::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if let Some((city, temp)) = line.split_once(';') {
            let city = city.trim().to_string();
            let temp = temp.trim().parse::<f64>().expect("Failed to parse temp");

            let stats = city_data.entry(city).or_insert_with(Stats::default);

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

    let mut sorted_city_data = BTreeMap::new();
    for (city, stats) in city_data {
        sorted_city_data.insert(city, stats);
    }

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
