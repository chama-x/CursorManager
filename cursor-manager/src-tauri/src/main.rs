// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cursor_manager::run;

fn main() {
    // Initialize the Tauri application
    run();
}
