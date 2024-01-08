mod client;
mod config;
mod daerah;
mod jadwal;

use chrono::NaiveDate;
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

    let jadwal = jadwal::load_jadwal(daerah, date).await;

    let nearest = jadwal::get_nearest(jadwal.items, date);

    let (next, prev) = match nearest {
        Ok((b, a)) => (a, b),
        Err(err) => {
            eprintln!("Cannot get nearest {:?}", err);
            std::process::exit(1);
        }
    };

    match prev {
        Some(jadwal) => {
            print!("{} {} ({} minutes ago), ", jadwal.name, jadwal.date, -jadwal.distance_from_now.unwrap_or(0));
        },
        None => {}
    };

    match next {
        Some(jadwal) => {
            println!("{} {} (in {} minutes)", jadwal.name, jadwal.date, jadwal.distance_from_now.unwrap_or(0));
        },
        None => {}
    };

    Ok(())
}
