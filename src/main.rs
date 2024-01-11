mod client;
mod config;
mod daerah;
mod jadwal;

use std::env;

use chrono::{Local, NaiveDate};
use clap::{Arg, ArgAction, ArgMatches, Command};

macro_rules! get_result_or_quit {
    ($result:expr, $err_msg:expr, $exit_code:expr) => {
        match $result {
            Ok(value) => value,
            Err(_) => {
                eprintln!("{}", $err_msg);
                std::process::exit($exit_code);
            }
        }
    };
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
    };
}

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

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let app = create_command();
    let matches = app.get_matches();

    let date = match matches.get_one::<String>("date") {
        Some(val) => NaiveDate::parse_from_str(&val, "%Y-%m-%d").expect("failed parse date"),
        None => Local::now().date_naive(),
    };

    let provinsi = get_result_or_quit!(
        get_provinsi(&matches),
        "provinsi not provided either in argument or as env JADWAL_PROVINSI",
        255
    )
    .to_uppercase();

    let kabupaten = get_result_or_quit!(
        get_kabupaten(&matches),
        "kabupaten not provided either in argument or as env(JADWAL_KABUPATEN)",
        255
    )
    .to_uppercase();

    let vec_daerah =
        get_result_or_quit!(daerah::load_daerah().await, "Error while load daerah", 255);

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
    let (prevs, nexts) = jadwal::get_prev_next(&jadwal.items, date);
    let prev = prevs.last();
    let (prev_name, prev_text) = match prev {
        Some(val) => (
            val.1,
            format!("{} {} <- {} minutes ago", val.1, val.2, -val.0),
        ),
        None => ("", "".to_string()),
    };

    let next = nexts.first();
    let (next_name, next_text) = match next {
        Some(val) => (
            val.1,
            format!("{} {} -> in {} minutes", val.1, val.2, val.0),
        ),
        None => ("", "".to_string()),
    };

    let all_day = matches.get_one::<bool>("all-day").unwrap_or(&false);
    if *all_day {
        println!(
            "Jadwal Sholat {} {} at {}",
            daerah.kabupaten, daerah.provinsi, date
        );
        for item in prevs {
            if prev_name == item.1 {
                println!("{}", prev_text);
            } else {
                println!("{} {}", item.1, item.2);
            };
        }

        for item in nexts {
            if next_name == item.1 {
                println!("{}", next_text);
            } else {
                println!("{} {}", item.1, item.2);
            };
        }
        return Ok(());
    }

    println!("{} {}", prev_text, next_text);

    Ok(())
}
