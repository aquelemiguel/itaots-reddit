use std::{env, time::Duration};

use chrono::Local;
use itertools::Itertools;
use roux::{inbox::InboxData, response::BasicThing, util::RouxError, Me, Reddit};
use serde_json::Value;
use tokio::time::sleep;

/// Returns a query string for the album title by adding wildcards for each letter.
/// Non-alphanumeric characters are removed from the final string.
///
/// # Arguments
/// * `acronym` - A string slice that holds the user provided acronym
///
/// # Examples
/// ```
/// let query = build_query("V:D:C.");
/// println!("{}", query);  // V* D* C*
/// ```
fn build_query(acronym: &str) -> String {
    acronym
        .chars()
        .filter_map(|c| (c.is_alphanumeric() || c.is_whitespace()).then_some(format!("{}* ", c)))
        .collect::<String>()
        .trim()
        .to_string()
}

fn rule_match_starting(title: &str, acronym: &str) -> bool {
    let title = title
        .to_uppercase()
        .split(' ')
        .map(|c| c.chars().next().unwrap())
        .collect::<String>();
    acronym == title
}

fn parse(results: &[(String, String)], acronym: &str) -> Vec<(String, String)> {
    results
        .iter()
        .filter(|(_, album)| rule_match_starting(album, acronym))
        .cloned()
        .collect()
}

async fn search(api_key: &str, query: &str) -> Vec<(String, String)> {
    let base_url = String::from("http://ws.audioscrobbler.com/2.0/");

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
        .collect_vec()
}

async fn auth_reddit() -> Result<Me, RouxError> {
    let reddit = Reddit::new(
        "macos:roux:v2.0.0 (by /u/blinkroot)",
        &env::var("REDDIT_CLIENT_ID").unwrap(),
        &env::var("REDDIT_CLIENT_SECRET").unwrap(),
    );

    let reddit = reddit
        .username(&env::var("REDDIT_USER_USERNAME").unwrap())
        .password(&env::var("REDDIT_USER_PASSWORD").unwrap());

    reddit.login().await
}

async fn handle(me: &Me, message: &BasicThing<InboxData>) {
    let body = &message.data.body;
    let lastfm_api_key = env::var("LASTFM_API_KEY").expect("lastfm api key is required");

    let mut bulletpoints: Vec<String> = vec![];

    for acronym in body.split(' ').skip(1).map(|s| s.to_string()) {
        let albums = search(&lastfm_api_key, &build_query(&acronym)).await;
        let albums = parse(&albums, &acronym);

        if let Some(picked) = albums.first() {
            let (artist, album) = picked;
            let bullet = format!("💿 **{acronym}** is **{album}** by {artist}");
            bulletpoints.push(bullet);
        }
    }

    let reply = bulletpoints.join("  \n\n");
    me.comment(&reply, &message.data.name).await.unwrap();

    // set as read so we won't reply again in the next loop
    me.mark_read(&message.data.name).await.unwrap();
}

fn log(emoji: char, msg: &str) {
    let ts = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    println!("{ts} | {emoji} {msg}");
}

#[tokio::main]
async fn main() {
    // register environment vars from .env
    dotenv::dotenv().ok();

    // authenticate to reddit
    let me = auth_reddit().await.unwrap();

    // wait for new mentions
    loop {
        log('🔎', "checking inbox for unread messages...");

        if let Ok(inbox) = me.unread().await {
            for message in inbox.data.children {
                log('🚨', "found a new message!");
                handle(&me, &message).await;
            }
        }
        sleep(Duration::from_secs(30)).await;
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{build_query, rule_match_starting, search};

    #[test]
    fn test_build_query() {
        assert_eq!(build_query("V:D:C."), "V* D* C*");
        assert_eq!(build_query("ITAOTS"), "I* T* A* O* T* S*");
    }

    #[test]
    fn test_rule_match_starting() {
        assert!(rule_match_starting("Since I Left You", "SILY"));
        assert!(rule_match_starting("The Lonesome Crowded West", "TLCW"));
    }

    #[tokio::test]
    async fn test_search() {
        let api_key = env::var("LASTFM_API_KEY").unwrap();

        assert!(search(&api_key, "S* I* T* K* O* L*").await.contains(&(
            "Stevie Wonder".to_string(),
            "Songs in the Key of Life".to_string()
        )));

        assert!(search(&api_key, "A* F* U* T*").await.contains(&(
            "Black Country, New Road".to_string(),
            "Ants From Up There".to_string()
        )));

        assert!(search(&api_key, "L* Y* S* F* L* A* T* H*")
            .await
            .contains(&(
                "Godspeed You! Black Emperor".to_string(),
                "Lift Your Skinny Fists Like Antennas to Heaven".to_string()
            )));
    }
}
