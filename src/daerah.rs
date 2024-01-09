use crate::{client, config};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Error, Write};

static URI_KABUPATEN: &'static str = "https://bimasislam.kemenag.go.id/ajax/getKabkoshalat";
static URI_JADWALSHALAT: &'static str = "https://bimasislam.kemenag.go.id/jadwalshalat";

#[derive(Serialize, Deserialize, Debug)]
pub struct Daerah {
    pub provinsi: String,
    pub provinsi_token: String, // x
    pub kabupaten: String,
    pub kabupaten_token: String, // y
}

async fn fetch_daerah(client: &Client) -> Vec<Daerah> {
    let response = client
        .get(URI_JADWALSHALAT)
        .send()
        .await
        .expect("Failed load provinsi");
    let body = response.text().await.expect("Failed to read response body");
    let fragment = Html::parse_document(body.as_str());
    let select_selector = Selector::parse(r#"select[id="search_prov"]"#)
        .expect("Failed parse selector for search_prov");
    let option_selector = Selector::parse("option").expect("Failed parse selector for option");

    let select = fragment
        .select(&select_selector)
        .next()
        .expect("Failed get select tag");
    // let values = select.select(&option_selector).into_iter().map(|option| { option.inner_html() });
    // println!("Values: {}", values.collect());
    let mut vec_daerah = Vec::new();
    for option in select.select(&option_selector) {
        let value = option.value();
        let provinsi_name = option.inner_html();
        let provinsi_token = value.attr("value").unwrap_or("");

        let daerah = build_daerah(client, &provinsi_name, &provinsi_token).await;
        vec_daerah.extend(daerah);
    }
    // println!("Vec {} {:?}", vec_daerah.len(), vec_daerah);

    vec_daerah
}

fn read_daerah_file() -> Result<Vec<Daerah>, std::io::Error> {
    let filename = config::get_cache_name("bimas-daerah.json");
    // println!("Filename daerah: {}", filename);
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let vec_daerah: Vec<Daerah> = serde_json::from_reader(reader)?;

    Ok(vec_daerah)
}

fn save_daerah_file(vec_daerah: &Vec<Daerah>) -> Result<(), std::io::Error> {
    let filename = config::get_cache_name("bimas-daerah.json");
    let mut file = File::create(filename)?;

    let json_string =
        serde_json::to_string_pretty(&vec_daerah).expect("Failed to serialize struct to JSON");

    file.write_all(json_string.as_bytes())
}

pub async fn load_daerah() -> Result<Vec<Daerah>, Error> {
    match read_daerah_file() {
        Ok(result) => Ok(result),
        Err(err) => {
            eprintln!(
                "Error read daerah file {:?}\n Downloading now...",
                err.kind()
            );

            let http_client = client::build_client().await.expect("Failed build client");
            let vec_daerah = fetch_daerah(&http_client).await;
            let _ = save_daerah_file(&vec_daerah);
            Ok(vec_daerah)
        }
    }
}

async fn build_daerah(client: &Client, provinsi_name: &str, provinsi_token: &str) -> Vec<Daerah> {
    // println!("Build daerah {}:", provinsi_name);
    let params = [("x", provinsi_token.to_string())];
    let response = client
        .post(URI_KABUPATEN)
        .form(&params)
        .send()
        .await
        .expect("Failed load kabupaten");

    let body = response.text().await.expect("Failed to read response body");
    let fragment = Html::parse_fragment(body.as_str());
    let option_selector = Selector::parse("option").expect("Failed parse selector for option");

    let mut daerahs = Vec::new();

    for option in fragment.select(&option_selector) {
        let value = option.value();
        let daerah = Daerah {
            provinsi: provinsi_name.to_string(),
            provinsi_token: provinsi_token.to_string(),
            kabupaten: option.inner_html().to_string(),
            kabupaten_token: value.attr("value").unwrap_or("").to_string(),
        };
        daerahs.push(daerah);
    }

    daerahs
}

pub fn list_daerah(vec_daerah: Vec<Daerah>) {
    vec_daerah.iter().for_each(|daerah| {
        println!(
            "Kabupaten: {}, Provinsi: {}",
            daerah.kabupaten, daerah.provinsi
        )
    });
}
