use bincode;
use glob::glob;
use js_parser;
use num_cpus;
use rayon::prelude::*;
use std;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use types::{HasteMap, SourceFile};
use utils;

pub const CACHE_DIR: &str = "cache";

pub fn derive_haste_map(project_path: PathBuf) -> HasteMap {
    let glob_time = Instant::now();
    let js_glob = format!("{}/{}", project_path.display(), "*/**/*.js");
    let parse_time = Instant::now();
    let file_vec: Vec<PathBuf> = glob(&js_glob)
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect();
    println!(
        "found files in: {}",
        utils::duration_to_ms(glob_time.elapsed())
    );
    let source_files: HasteMap = file_vec
        .par_iter()
        .map(|path| (path.clone(), File::open(&path)))
        .filter(|tuple| tuple.1.is_ok())
        .map(|(path, open)| (path, open.unwrap()))
        .map(|(path, mut file)| {
            let mut content = String::new();
            (path, file.read_to_string(&mut content), content)
        })
        .filter(|tuple| tuple.1.is_ok())
        .map(|(path, _, content)| SourceFile {
            path,
            dependencies: js_parser::parse(&content),
        })
        .collect();
    println!(
        "Parsed files in: {}",
        utils::duration_to_ms(parse_time.elapsed())
    );
    source_files
}

pub fn read_haste_map_from_cache() -> HasteMap {
    let cache_glob = format!("{}/{}", CACHE_DIR, "*");
    let paths: Vec<PathBuf> = glob(&cache_glob)
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect();
    let haste: Vec<SourceFile> = paths
        .par_iter()
        .flat_map(|path| -> Vec<SourceFile> {
            let mut cache = File::open(path).unwrap();
            let mut buf = vec![];
            cache.read_to_end(&mut buf).unwrap();
            bincode::deserialize(&buf).unwrap()
        })
        .collect();
    haste
}

fn make_cache_filename(id: usize) -> String {
    format!("{}/{}.bincode", CACHE_DIR, id)
}

pub fn write_to_cache(haste_map: &HasteMap) {
    let threads = num_cpus::get();
    // let threads = 1;
    let min: usize = 1;
    let chunk_size: usize = haste_map.len() / threads;
    let chunk_size = std::cmp::max(chunk_size, min);
    let mut counter = 1..chunk_size + 1;
    println!("Writing cache in {} chunks of size {}", threads, chunk_size);
    let chunks = haste_map.chunks(chunk_size);
    std::fs::create_dir_all(CACHE_DIR).unwrap();
    for chunk in chunks {
        let cache_file_name = make_cache_filename(counter.next().unwrap());
        println!("chunk: {}", cache_file_name);
        let serialized = bincode::serialize(chunk).unwrap();
        let mut cache = File::create(cache_file_name).expect("Failed to open cache.bincode");
        cache
            .write_all(&serialized)
            .expect("Failed to write cache.json");
    }
}
