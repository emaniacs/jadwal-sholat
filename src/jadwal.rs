use crate::{client, config, daerah};
use chrono::{Datelike, NaiveDate};
use reqwest::Client;
use serde_json;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;

static URI_JADWALSHALAT_X: &'static str = "https://bimasislam.kemenag.go.id/ajax/getShalatbln";

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

// async fn load_jadwal(daerah: &Daerah, bulan: &str) -> Vec<Jadwal> {
pub async fn load_jadwal(daerah: &daerah::Daerah, date: NaiveDate) -> Value {
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

    vec_jadwal
}
