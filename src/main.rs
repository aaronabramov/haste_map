extern crate bincode;
extern crate glob;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate rayon;
extern crate regex;
#[macro_use]
extern crate serde_derive;

use glob::glob;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

mod js_parser;

const CACHE_DIR: &str = "cache";

#[derive(Serialize, Deserialize, Debug)]
struct SourceFile {
    path: PathBuf,
    dependencies: Vec<String>,
}

type HasteMap = Vec<SourceFile>;

fn get_project_path() -> PathBuf {
    let passed_path = env::args()
        .nth(1)
        .expect("First argument should be a path.");
    let project_path = PathBuf::from(passed_path);
    fs::canonicalize(project_path).expect("Path not found")
}

fn derive_haste_map(project_path: PathBuf) -> HasteMap {
    let glob_time = Instant::now();
    let js_glob = format!("{}/{}", project_path.display(), "*/**/*.js");
    let parse_time = Instant::now();
    let file_vec: Vec<PathBuf> = glob(&js_glob)
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect();
    println!("found files in: {}", duration_to_ms(glob_time.elapsed()));
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
    println!("Parsed files in: {}", duration_to_ms(parse_time.elapsed()));
    source_files
}

fn read_haste_map_from_cache() -> Vec<SourceFile> {
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

fn write_to_cache(haste_map: &HasteMap) {
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

// Generate a deterministic string version of a haste map. Used for comparing output to JS implementation.
fn get_deterministic_hastemap_string(haste_map: HasteMap) -> String {
    let mut lines: Vec<String> = haste_map
        .par_iter()
        .map(|source| {
            let mut dep: Vec<String> = source.dependencies.clone();
            dep.sort();
            format!("{}|{}", source.path.display(), dep.join("|"))
        })
        .collect();

    lines.sort();
    lines.join("\n")
}

fn main() {
    let now = Instant::now();
    let cache_path = Path::new(CACHE_DIR);
    let use_cache = cache_path.exists();
    match use_cache {
        true => println!("Found cache: {}", cache_path.display()),
        false => println!("no cache found. Recalculating..."),
    }
    let haste_map = if use_cache {
        read_haste_map_from_cache()
    } else {
        let path = get_project_path();
        let haste_map = derive_haste_map(path);
        // println!("{:?}", haste_map);
        write_to_cache(&haste_map);
        haste_map
    };
    println!("hastmap has {} files", haste_map.len());
    let elapsed = now.elapsed();
    println!("ms: {}", duration_to_ms(elapsed));

    const COMPARISON_ARTIFACT: &str = "haste_map_rs.txt";
    let comparison_artifact = Path::new(COMPARISON_ARTIFACT);
    let generate_artifact = comparison_artifact.exists();
    match generate_artifact {
        true => println!(
            "Found comparison artifact file: {}",
            comparison_artifact.display()
        ),
        false => println!("no comparison artifact found. Recalculating..."),
    }
    if !generate_artifact {
        let mut haste_map_txt =
            File::create(COMPARISON_ARTIFACT).expect("Failed to open cache.bincode");
        haste_map_txt
            .write_all(get_deterministic_hastemap_string(haste_map).as_bytes())
            .expect("Failed to write haste_map.txt");
    }
}

fn duration_to_ms(d: std::time::Duration) -> f64 {
    (d.as_secs() as f64) * 1000.0 + (f64::from(d.subsec_nanos()) / 1000_000.0)
}
