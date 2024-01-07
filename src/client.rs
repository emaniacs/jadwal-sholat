use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Error};

static HOMEPAGE: &'static str = "https://bimasislam.kemenag.go.id";

pub async fn build_client() -> Result<Client, Error> {
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
    // println!("Build client");

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
