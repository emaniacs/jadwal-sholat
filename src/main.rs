use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;

static HOMEPAGE: &'static str = "https://bimasislam.kemenag.go.id";
static URI_JADWALSHALAT: &'static str = "https://bimasislam.kemenag.go.id/jadwalshalat";
static URI_KABUPATEN: &'static str = "https://bimasislam.kemenag.go.id/ajax/getKabkoshalat";
static DAERAH_JSON_FILE: &'static str = "/tmp/bimas-daerah.json";

#[derive(Serialize, Deserialize, Debug)]
struct Daerah {
    provinsi: String,
    provinsi_token: String, // x
    kabupaten: String,
    kabupaten_token: String, // y
}

async fn build_client() -> Result<Client, Error> {
    let mut headers_map = HeaderMap::new();
    let headers = [
        ("Accept", "*/*,"),
        ("Accept-Language", "en-US,en;q=0.8"),
        ("Origin", "https://bimasislam.kemenag.go.id"),
        ("Referer", "https://bimasislam.kemenag.go.id/jadwalshalat"),
        ("Sec-Fetch-Dest", "empty"),
        ("Sec-Fetch-Mode", "cors"),
        ("Sec-Fetch-Site", "same-origin"),
        ("Sec-GPC", "1"),
        ("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36"),
        ("X-Requested-With", "XMLHttpRequest"),
        ("sec-ch-ua", "'Brave';v='117', 'Not;A=Brand';v='8', 'Chromium';v='117'"),
        ("sec-ch-ua-platform", "Linux"),
    ];
    for header in headers {
        headers_map.insert(header.0, HeaderValue::from_static(header.1));
    }
    let client = Client::builder()
        .cookie_store(true)
        .default_headers(headers_map)
        .build()
        .unwrap();
    println!("Build client");

    let response = client
        .get(HOMEPAGE)
        .send()
        .await
        .expect("Failed build client at HOMEPAGE");

    if !response.status().is_success() {
        eprintln!(
            "Build client to {} got error {}",
            HOMEPAGE,
            response.status()
        );
        std::process::exit(1);
    }

    // let body_bytes = response.bytes().await?;
    // let _ = std::fs::write("/tmp/bimas.html", body_bytes);

    Ok(client)
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
    println!("Vec {} {:?}", vec_daerah.len(), vec_daerah);

    vec_daerah
}

fn read_daerah_file() -> Result<Vec<Daerah>, serde_json::Error> {
    let file = File::open(DAERAH_JSON_FILE).expect("Cant open daerah file");
    let reader = BufReader::new(file);

    let vec_daerah: Vec<Daerah> = serde_json::from_reader(reader)?;

    Ok(vec_daerah)
}

fn save_daerah_file(vec_daerah: &Vec<Daerah>) -> Result<(), std::io::Error> {
    let mut file = File::create(DAERAH_JSON_FILE)?;

    let json_string =
        serde_json::to_string_pretty(&vec_daerah).expect("Failed to serialize struct to JSON");

    file.write_all(json_string.as_bytes())
}

async fn load_daerah() -> Vec<Daerah> {
    let vec_daerah = match read_daerah_file() {
        Ok(result) => result,
        Err(err) => {
            println!("Error read daerah file {:?}", err);
            let client = build_client().await.expect("Failed build client");
            let vec_daerah = fetch_daerah(&client).await;
            let _ = save_daerah_file(&vec_daerah);
            vec_daerah
        }
    };

    vec_daerah
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let vec_daerah = load_daerah().await;
    println!("Daerah {:#?}", vec_daerah);
    Ok(())
}