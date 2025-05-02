// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod model;
mod repositories;
mod service;

use crate::core::kasuri::Kasuri;

fn main() {
    if let Err(e) = Kasuri::new().and_then(|k| k.run()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
