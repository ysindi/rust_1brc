use std::{
    fs::File,
    time::Instant,
};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use memmap2::Mmap;
use memchr::memchr;
use fast_float;

#[derive(Debug, Clone)]
struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            count: 0,
            sum: 0.0,
        }
    }
}

impl Stats {
    fn update(&mut self, v: f64) {
        self.min = self.min.min(v);
        self.max = self.max.max(v);
        self.count += 1;
        self.sum += v;
    }

    fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.count += other.count;
        self.sum += other.sum;
    }
}

fn process_chunk(chunk: &[u8]) -> FxHashMap<&[u8], Stats> {
    let mut local_data: FxHashMap<&[u8], Stats> = Default::default();
    
    for line in chunk.split(|&byte| byte == b'\n') {
        let mut parts = line.splitn(2, |&byte| byte == b';');
        if let (Some(city_bytes), Some(temp_bytes)) = (parts.next(), parts.next()) {
            let temp = fast_float::parse(temp_bytes).unwrap();
            local_data.entry(&city_bytes).or_default().update(temp);
        }
    }
    
    local_data
}

fn main() {
    let start_time = Instant::now();

    let file = File::open("../measurements.txt").expect("Error while trying to open file");
    let mmap = unsafe { Mmap::map(&file).expect("Failed to map the file") };
    
    let num_threads = rayon::current_num_threads().max(1);
    let chunk_size = mmap.len() / num_threads;
    
    let mut chunks: Vec<(usize, usize)> = vec![];
    let mut start = 0;
    for _ in 0..num_threads {
        let end = (start + chunk_size).min(mmap.len());

        let adjusted_end = if end < mmap.len() {
            let next_new_line = memchr(b'\n', &mmap[end..]).unwrap_or(0);
            end + next_new_line
        } else {
            end
        };

        chunks.push((start, adjusted_end));
        start = adjusted_end + 1;

        if start >= mmap.len() {
            break;
        }
    }
    
    let results: Vec<FxHashMap<&[u8], Stats>> = chunks
        .par_iter()
        .map(|&(start, end)| process_chunk(&mmap[start..end]))
        .collect();
    
    let mut city_data = FxHashMap::default();
    for local_data in results {
        for (city, local_stats) in local_data {
            city_data
                .entry(city)
                .or_insert_with(Stats::default)
                .merge(&local_stats);
        }
    }

    let mut sorted_city_data: Vec<_> = city_data.into_iter().collect();
    sorted_city_data.par_sort_unstable_by_key(|(city, _)| *city);
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
                std::str::from_utf8(city).unwrap(),
                stats.min,
                average,
                stats.max
            );
        }
    }
    println!("}}");

    let duration = start_time.elapsed();
    println!("Time taken: {:.3} seconds", duration.as_secs_f64());
}