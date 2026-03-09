mod controllers;
mod enums;
mod models;
mod parsers;
mod repositories;
mod services;
mod views;

use repositories::file_manager_repository;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_path = args
        .get(1)
        .map(|name| file_manager_repository::normalize_cpad_path(name));

    if let Err(e) = controllers::app_controller::run(file_path) {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}
