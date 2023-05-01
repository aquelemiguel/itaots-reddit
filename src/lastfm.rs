use std::env;

use serde_json::Value;

pub fn parse(results: &[(String, String)], acronym: &str) -> Vec<(String, String)> {
    results
        .iter()
        .filter(|(_, album)| {
            album
                .to_uppercase()
                .split(' ')
                .map(|c| c.chars().next().unwrap())
                .collect::<String>()
                == acronym
        })
        .cloned()
        .collect()
}

pub async fn search(query: &str) -> Vec<(String, String)> {
    let base_url = String::from("http://ws.audioscrobbler.com/2.0/");
    let api_key = env::var("LASTFM_API_KEY").expect("lastfm api key is required");

    let url = format!(
        "{}?method=album.search&album={}&api_key={}&format=json",
        base_url, query, api_key
    );

    let res = reqwest::get(url).await.unwrap().text().await.unwrap();
    let res = serde_json::from_str::<Value>(&res).unwrap();

    res["results"]["albummatches"]["album"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["artist"].as_str().unwrap().trim().to_string(),
                r["name"].as_str().unwrap().trim().to_string(),
            )
        })
        .collect()
}
