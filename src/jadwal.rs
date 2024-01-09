use crate::{client, config, daerah};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use reqwest::Client;
use serde_json;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;

static URI_JADWALSHALAT_X: &'static str = "https://bimasislam.kemenag.go.id/ajax/getShalatbln";

#[derive(Debug)]
pub struct JadwalSholat {
    pub name: String,
    pub date: String,
    pub distance_from_now: Option<i64>,
}

#[derive(Debug)]
pub struct Jadwal {
    pub tanggal: String,
    pub items: Vec<JadwalSholat>,
}

async fn fetch_jadwal(client: &Client, daerah: &daerah::Daerah, date: &NaiveDate) -> Value {
    let params = [
        ("x", daerah.provinsi_token.to_string()),
        ("y", daerah.kabupaten_token.to_string()),
        ("bln", date.month().to_string()),
        ("thn", date.year().to_string()),
    ];

    let response = client
        .post(URI_JADWALSHALAT_X)
        .form(&params)
        .send()
        .await
        .expect("Failed load kabupaten");

    let json = response
        .json::<Value>()
        .await
        .expect("Failed read jadwal sholat");
    let value = &json["data"];
    value.clone()
}

fn generate_jadwal_filename(daerah: &daerah::Daerah, bulan: &str) -> String {
    let provinsi = daerah.provinsi.to_lowercase();
    let kabupaten = daerah.kabupaten.to_lowercase();

    let filename = format!("{}-{}-{}.json", &provinsi, &kabupaten, &bulan);

    return config::get_cache_name(&filename);
}

fn read_jadwal_file(daerah: &daerah::Daerah, bulan: &str) -> Result<Value, std::io::Error> {
    let filename = generate_jadwal_filename(&daerah, &bulan);
    let file = File::open(filename)?;
    // eprintln!("file {:?}", file);

    let reader = BufReader::new(file);
    let vec_jadwal: Value = serde_json::from_reader(reader)?;

    Ok(vec_jadwal)
}

fn save_jadwal_file(
    daerah: &daerah::Daerah,
    bulan: &str,
    jadwals: &Value,
) -> Result<(), std::io::Error> {
    let filename = generate_jadwal_filename(&daerah, &bulan);
    let mut file = File::create(filename)?;

    let json_string =
        serde_json::to_string_pretty(&jadwals).expect("Failed to serialize struct to JSON");

    file.write_all(json_string.as_bytes())
}

// fn extract_jadwal(object: Value) -> Jadwal {
//     let items = vec![];
//     for (key, val)  in object {
//         if key == "tanggal" {
//             items.append((key, val));
//         }
//     }
//     Jadwal {
//         tanggal: object["tanggal"],
//         items: items
//     }
// }

// async fn load_jadwal(daerah: &Daerah, bulan: &str) -> Vec<Jadwal> {
pub async fn load_jadwal(daerah: &daerah::Daerah, date: NaiveDate) -> Jadwal {
    let bulan = date.format("%Y-%m").to_string();
    let vec_jadwal = match read_jadwal_file(&daerah, &bulan) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error read jadwal file {:?}", err.kind());

            let client = client::build_client().await.expect("Failed build client");
            let value = fetch_jadwal(&client, &daerah, &date).await;
            let _ = save_jadwal_file(&daerah, &bulan, &value);
            value
        }
    };

    let hari = date.format("%Y-%m-%d").to_string();

    let jadwal_obj = match &vec_jadwal[&hari] {
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

    let mut items = vec![];

    for (key, value) in jadwal_obj.into_iter() {
        if key == "tanggal" {
            continue;
        }
        let val = match value {
            Value::String(v) => v,
            _ => "",
        };

        items.push(JadwalSholat {
            name: key.to_string(),
            date: String::from(val),
            distance_from_now: None,
        });
    }

    Jadwal {
        tanggal: jadwal_obj["tanggal"].to_string(),
        items,
    }
}

pub fn get_prev_next(
    items: Vec<JadwalSholat>,
    date: NaiveDate,
) -> (Vec<(i64, String, String)>, Vec<(i64, String, String)>) {
    let now = Local::now();

    let hari = date.format("%Y-%m-%d").to_string();
    let date_format = "%Y-%m-%d %H:%M";
    let mut prevs: Vec<(i64, String, String)> = vec![];
    let mut nexts: Vec<(i64, String, String)> = vec![];
    for item in items {
        let date_str = format!("{} {}", hari, item.date);
        let dt = NaiveDateTime::parse_from_str(&date_str, &date_format);
        let diff = (dt.expect("Can parse time").time() - now.time()).num_minutes();

        if diff >= 0 {
            nexts.push((diff, item.name, item.date));
        } else {
            prevs.push((diff, item.name, item.date));
        }
    }
    nexts.sort();
    prevs.sort();

    (prevs, nexts)
}

#[derive(Debug)]
pub struct SortJadwalResult(Vec<(i64, String, String)>, Vec<(i64, String, String)>);

pub fn sort_jadwal(
    items: &Vec<JadwalSholat>,
    date: NaiveDate,
) -> SortJadwalResult {
    let now = Local::now();

    let hari = date.format("%Y-%m-%d").to_string();
    let date_format = "%Y-%m-%d %H:%M";
    let mut prevs: Vec<(i64, String, String)> = vec![];
    let mut nexts: Vec<(i64, String, String)> = vec![];
    for item in items {
        let date_str = format!("{} {}", hari, item.date);
        let dt = NaiveDateTime::parse_from_str(&date_str, &date_format);
        let diff = (dt.expect("Can parse time").time() - now.time()).num_minutes();

        if diff >= 0 {
            nexts.push((diff, item.name.clone(), item.date.clone()));
        } else {
            prevs.push((diff, item.name.clone(), item.date.clone()));
        }
    }
    nexts.sort();
    prevs.sort();
    SortJadwalResult(prevs, nexts)
}
