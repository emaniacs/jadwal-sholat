mod client;
mod config;
mod daerah;
mod jadwal;

use chrono::{Local, NaiveDate};
use std::env;
use clap::Parser;

#[derive(Parser, Debug, Default)]
#[clap(author="Author Name", version, about="A Very simple Package Hunter")]
struct Args {
    date: Option<String>,

    /// Name of the person to greet
    #[arg(short, long)]
    provinsi: Option<String>,

    /// Name of the person to greet
    #[arg(short, long)]
    kabupaten: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();

    let date: NaiveDate = match args.date {
        Some(val) => NaiveDate::parse_from_str(&val, "%Y-%m-%d").expect("failed parse date"),
        None => Local::now().date_naive(),
    };

    let provinsi: String = match args.provinsi {
        Some(val) => val.to_uppercase(),
        None => "DKI JAKARTA".to_string(),
    };

    let kabupaten: String = match args.kabupaten {
        Some(val) => val.to_uppercase(),
        None => "KOTA JAKARTA".to_string(),
    };

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
