mod controllers;
mod services;
mod views;
mod models;
mod repositories;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_path = args.get(1).map(|name| {
        if name.ends_with(".cpad") {
            name.clone()
        } else {
            format!("{}.cpad", name)
        }
    });
    controllers::app_controller::run(file_path).unwrap();
}
