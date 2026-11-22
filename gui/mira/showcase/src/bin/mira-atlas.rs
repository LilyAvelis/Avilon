#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mira::Edition;

fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() == Some("--style")
        && let Some(edition) = args.next().as_deref().and_then(edition)
    {
        return mira::run(edition);
    }
    mira::run_atlas()
}

fn edition(name: &str) -> Option<Edition> {
    Some(match name {
        "lumen" => Edition::Lumen,
        "verstka" => Edition::Verstka,
        "contact" => Edition::Contact,
        "azimuth" => Edition::Azimuth,
        "herbarium" => Edition::Herbarium,
        "pult" => Edition::Pult,
        "orbit" => Edition::Orbit,
        "lozha" => Edition::Lozha,
        "kalka" => Edition::Kalka,
        "semaphore" => Edition::Semaphore,
        _ => return None,
    })
}
