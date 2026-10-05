#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args
        .iter()
        .find(|a| a.starts_with("--mode="))
        .map(|a| a.split('=').nth(1).unwrap_or("desktop"))
        .unwrap_or("desktop");
    let port = args
        .iter()
        .find(|a| a.starts_with("--port="))
        .and_then(|a| a.split('=').nth(1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(14200);
    // Optional override for hermetic smoke tests; defaults to the app data dir.
    let data_dir = args
        .iter()
        .find(|a| a.starts_with("--data-dir="))
        .map(|a| a.split('=').nth(1).unwrap_or("").to_string());

    // Allow "web" as substring (e.g., --mode=web, --mode web)
    let is_web = mode == "web";
    let is_both = mode == "both";

    if is_web {
        knowtequiz_lib::run_web_server(port, data_dir);
    } else if is_both {
        knowtequiz_lib::run_both(port, data_dir);
    } else {
        knowtequiz_lib::run_desktop();
    }
}
