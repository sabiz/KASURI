// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod model;
mod repositories;
mod service;

fn main() {
    core::log::init_logger();
    if let Err(e) = core::kasuri_app::run() {
        log::error!("Kasuri error: {}", e);
        std::process::exit(1);
    }
}
