pub mod lastfm;
pub mod reddit;
pub mod utils;

use std::time::Duration;

use lastfm::{parse, search};
use reddit::{auth, fetch_unread, parse_request, reply};
use tokio::time::sleep;
use utils::{build_query, format_bulletpoint, get_footer};

#[tokio::main]
async fn main() {
    // register environment vars from .env
    dotenv::dotenv().ok();

    // authenticate to reddit
    let me = auth().await.expect("failed to authenticate with reddit");

    // wait for new mentions
    loop {
        if let Ok(messages) = fetch_unread(&me).await {
            for message in messages.iter() {
                let mut bulletpoints = String::new();

                for acronym in parse_request(message).iter() {
                    let albums = search(&build_query(acronym)).await;
                    let albums = parse(&albums, acronym);

                    if let Some(picked) = albums.first() {
                        let bullet = format!("{}  \n\n", format_bulletpoint(picked, acronym));
                        bulletpoints.push_str(&bullet);
                    }
                }

                let content = format!("{}\n\n{}", bulletpoints, &get_footer());
                reply(&me, &message.data.name, &content).await;
            }
        }

        sleep(Duration::from_secs(30)).await;
    }
}
