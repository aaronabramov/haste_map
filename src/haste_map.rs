use bincode;
use glob::glob;
use js_parser;
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
    utils::log_time(glob_time, &"found files");
    let chunk_size = utils::get_chunk_size(&file_vec);
    // make more chunks, so they're spread more evenly and the slowest
    // chunk doesn't hold the last thread for too long. There's still
    // a lot of room for optimization. (e.g. shared pool of files to parse)
    let chunks: Vec<&[PathBuf]> = file_vec.chunks(chunk_size / 10).collect();
    let source_files: HasteMap = chunks
        .par_iter()
        .flat_map(|chunk| js_parser::parse_chunk(chunk))
        .collect();
    utils::log_time(parse_time, &"parsed files");
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
    let chunk_size = utils::get_chunk_size(haste_map);
    println!("Writing cache in chunks of size {}", chunk_size);
    let chunks = haste_map.chunks(chunk_size);
    std::fs::create_dir_all(CACHE_DIR).unwrap();
    for (counter, chunk) in chunks.enumerate() {
        let cache_file_name = make_cache_filename(counter);
        println!("chunk: {}", cache_file_name);
        let serialized = bincode::serialize(chunk).unwrap();
        let mut cache = File::create(cache_file_name).expect("Failed to open cache.bincode");
        cache
            .write_all(&serialized)
            .expect("Failed to write cache.json");
    }
}
