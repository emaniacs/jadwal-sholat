mod client;
mod config;
mod daerah;
mod jadwal;

use std::env;

use chrono::{Local, NaiveDate};
use clap::Parser;

#[derive(Parser, Debug, Default)]
#[command(author, version, about)]
struct Args {
    date: Option<String>,

    #[arg(short, long)]
    provinsi: Option<String>,

    #[arg(short, long)]
    kabupaten: Option<String>,

    #[arg(long)]
    list_daerah: bool,

    #[arg(long)]
    all_day: bool,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();

    let date = match args.date {
        Some(val) => NaiveDate::parse_from_str(&val, "%Y-%m-%d").expect("failed parse date"),
        None => Local::now().date_naive(),
    };

    let provinsi: String = match args.provinsi {
        Some(val) => val.to_uppercase(),
        None => env::var("JADWAL_PROVINSI")
            .expect("provinsi not provided in option or env, please set JADWAL_PROVINSI")
            .to_uppercase(),
    };

    let kabupaten: String = match args.kabupaten {
        Some(val) => val.to_uppercase(),
        None => env::var("JADWAL_KABUPATEN")
            .expect("provinsi not provided in option or env, please set JADWAL_KABUPATEN")
            .to_uppercase(),
    };

    let vec_daerah = daerah::load_daerah().await;
    if args.list_daerah {
        daerah::list_daerah(vec_daerah);
        return Ok(());
    }

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
    if args.all_day {
        println!(
            "Jadwal Sholat {} {} at {}",
            daerah.kabupaten, daerah.provinsi, date
        );
        jadwal
            .items
            .iter()
            .for_each(|item| println!("{} {}", item.name, item.date));
        return Ok(());
    }

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
            print!(
                "{} {} ({} minutes ago), ",
                jadwal.name,
                jadwal.date,
                -jadwal.distance_from_now.unwrap_or(0)
            );
        }
        None => {}
    };

    match next {
        Some(jadwal) => {
            println!(
                "{} {} (in {} minutes)",
                jadwal.name,
                jadwal.date,
                jadwal.distance_from_now.unwrap_or(0)
            );
        }
        None => {}
    };

    Ok(())
}
