// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod model;
mod repositories;
mod service;

use crate::core::kasuri::Kasuri;

fn main() {
    core::log::init();
    if let Err(e) = Kasuri::new().and_then(|k| k.run()) {
        // log_error! マクロを直接使用
        log_error!("Kasuri error: {}", e);
        std::process::exit(1);
    }
}
