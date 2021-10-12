#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/built.rs"));
pub fn print_info_lala(name: &str) {
    let commit = GIT_VERSION.unwrap_or("NA");
    let dt = chrono::DateTime::parse_from_str(BUILT_TIME_UTC, "%a, %d %h %Y %T %z")
        .unwrap()
        .with_timezone(&chrono::Local);
    println!("application: {}", name);
    println!("version: {}", PKG_VERSION);
    println!("commit: {}", commit);
    println!("build date: {}", dt);
    println!("profile: {}", PROFILE);
    println!("features: {}", FEATURES_STR);
}
pub fn print_info_short(name: &str) {
    let commit = GIT_VERSION.unwrap_or("NA");
    let dt = chrono::DateTime::parse_from_str(BUILT_TIME_UTC, "%a, %d %h %Y %T %z")
        .unwrap()
        .with_timezone(&chrono::Local);
    println!("{}", name);
    println!("{} ({}, {})", PKG_VERSION, commit, dt.format("%a %d %h"));
}
