use std::{fs::File, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Create videos.db if it doesn't exist
    let db_path = Path::new("videos.db");
    if !db_path.exists() {
        File::create(db_path).expect("Failed to create videos.db");
        println!("Created videos.db");
    }
}
