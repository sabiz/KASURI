mod application;
mod fuzzy_sorter;
mod kasuri;
mod powershell;
mod settings;

use crate::kasuri::Kasuri;

fn main() {
    if let Err(e) = Kasuri::new().and_then(|k| k.run()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
