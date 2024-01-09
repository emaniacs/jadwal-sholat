mod client;
mod config;
mod daerah;
mod jadwal;

use std::env;

use chrono::{Local, NaiveDate};
use clap::{Arg, ArgAction, ArgMatches, Command};

fn create_command() -> Command {
    Command::new("jadwal-shalat")
        .arg(Arg::new("date").help("Date of jadwal shalat, default today, format YEAR-MONTH-DAY"))
        .arg(
            Arg::new("provinsi")
                .short('p')
                .long("provinsi")
                .required(false)
                .help("Provinsi, if not provided will read env JADWAL_PROVINSI "),
        )
        .arg(
            Arg::new("kabupaten")
                .short('k')
                .long("kabupaten")
                .required(false)
                .help("Kabupaten, if not provided will read env JADWAL_KABUPATEN "),
        )
        .arg(
            Arg::new("list-daerah")
                .long("list-daerah")
                .short('l')
                .action(ArgAction::SetTrue)
                .help("Show list daerah and exit"),
        )
        .arg(
            Arg::new("all-day")
                .short('a')
                .action(ArgAction::SetTrue)
                .long("all-day")
                .help("Show all jadwal in current date"),
        )
}

fn get_provinsi(matches: &ArgMatches) -> Result<String, env::VarError> {
    match matches.get_one::<String>("provinsi") {
        Some(val) => Ok(val.to_string()),
        None => env::var("JADWAL_PROVINSI"),
    }
}

fn get_kabupaten(matches: &ArgMatches) -> Result<String, env::VarError> {
    match matches.get_one::<String>("kabupaten") {
        Some(val) => Ok(val.to_string()),
        None => env::var("JADWAL_KABUPATEN"),
    }
}

fn quit(message: String, code: i32) {
    eprintln!("{}", message);
    std::process::exit(code);
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let app = create_command();
    let matches = app.get_matches();

    let date = match matches.get_one::<String>("date") {
        Some(val) => NaiveDate::parse_from_str(&val, "%Y-%m-%d").expect("failed parse date"),
        None => Local::now().date_naive(),
    };

    let provinsi: String = match get_provinsi(&matches) {
        Ok(val) => val.to_uppercase(),
        Err(_) => {
            eprintln!("provinsi not provided either in argument or as env JADWAL_PROVINSI");
            std::process::exit(255);
        }
    };

    let kabupaten: String = match get_kabupaten(&matches) {
        Ok(val) => val.to_uppercase(),
        Err(_) => {
            eprintln!("kabupaten not provided either in argument or as env JADWAL_KABUPATEN");
            std::process::exit(255);
        }
    };

    let vec_daerah = match daerah::load_daerah().await {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Error while load daerah {:?}", err);
            std::process::exit(255);
        }
    };

    let list_daerah = matches.get_one::<bool>("list-daerah").unwrap_or(&false);
    if *list_daerah {
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
            std::process::exit(255);
        }
    };

    let jadwal = jadwal::load_jadwal(daerah, date).await;
    let all_day = matches.get_one::<bool>("all-day").unwrap_or(&false);
    if *all_day {
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
