// Unter Windows als Fenster-App starten, ohne zusätzliches Konsolenfenster.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    shot_villa_lib::run();
}
