#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate bincode;
extern crate glob;
extern crate num_cpus;
extern crate rayon;
extern crate regex;

use rayon::prelude::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

mod haste_map;
mod js_parser;
mod types;
mod utils;

fn get_project_path() -> PathBuf {
    let passed_path = env::args()
        .nth(1)
        .expect("First argument should be a path.");
    let project_path = PathBuf::from(passed_path);
    fs::canonicalize(project_path).expect("Path not found")
}

// Generate a deterministic string version of a haste map. Used for comparing output to JS implementation.
fn get_deterministic_hastemap_string(haste_map: types::HasteMap) -> String {
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
    let cache_path = Path::new(haste_map::CACHE_DIR);
    let use_cache = cache_path.exists();
    match use_cache {
        true => println!("Found cache: {}", cache_path.display()),
        false => println!("no cache found. Recalculating..."),
    }
    let haste_map = if use_cache {
        haste_map::read_haste_map_from_cache()
    } else {
        let path = get_project_path();
        let haste_map = haste_map::derive_haste_map(path);
        // println!("{:?}", haste_map);
        haste_map::write_to_cache(&haste_map);
        haste_map
    };
    println!("hastmap has {} files", haste_map.len());
    let elapsed = now.elapsed();
    println!("ms: {}", utils::duration_to_ms(elapsed));

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
