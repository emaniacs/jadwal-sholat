mod client;
mod config;
mod daerah;
mod jadwal;

use chrono::NaiveDate;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();

    // Ensure there are exactly 2 arguments
    if args.len() != 4 {
        eprintln!("Usage: {} <date> <provinsi> <kabupaten>", args[0]);
        std::process::exit(1);
    }

    let date = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d").expect("failed parse date");
    let provinsi = args[2].to_uppercase().to_owned();
    let kabupaten = args[3].to_uppercase().to_owned();

    let vec_daerah = daerah::load_daerah().await;
    let daerah = match vec_daerah
        .iter()
        .find(|&item| item.provinsi == provinsi && item.kabupaten == kabupaten)
    {
        Some(result) => result,
        None => {
            eprintln!("Daerah '{} {}' not exist in file", provinsi, kabupaten);
            std::process::exit(1);
        }
    };

    // println!("Daerah: {:#?}", daerah);
    let hari = date.format("%Y-%m-%d").to_string();
    let jadwals = jadwal::load_jadwal(daerah, date).await;
    let jadwal = match &jadwals[&hari] {
        Value::Object(object) => object,
        Value::Null => {
            eprintln!("No jadwal for {}", hari);
            std::process::exit(1);
        }
        _ => {
            eprintln!("Invalid jadwal type");
            std::process::exit(1);
        }
    };

    println!(
        "Jadwal Sholat {} {} : {:#?}",
        daerah.kabupaten, daerah.provinsi, jadwal
    );
    Ok(())
}
