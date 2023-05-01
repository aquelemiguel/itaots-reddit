use roux::{inbox::InboxData, response::BasicThing, util::RouxError, Me, Reddit};
use std::env;

use crate::utils::log;

pub async fn reply(me: &Me, message_id: &str, content: &str) {
    if let Ok(_) = me.comment(&content, &message_id).await {
        me.mark_read(&message_id).await.unwrap();
    }
}

pub fn parse_request(message: &BasicThing<InboxData>) -> Vec<String> {
    let content = &message.data.body;
    content
        .split(' ')
        .skip(1)
        .map(|s| s.to_uppercase())
        .collect()
}

pub async fn fetch_unread(me: &Me) -> Result<Vec<BasicThing<InboxData>>, RouxError> {
    log('🔎', "checking inbox for unread messages...");
    me.unread().await.and_then(|inbox| Ok(inbox.data.children))
}

pub async fn auth() -> Result<Me, RouxError> {
    Reddit::new(
        "macos:roux:v2.0.0 (by u/blinkroot)",
        &env::var("REDDIT_CLIENT_ID").unwrap(),
        &env::var("REDDIT_CLIENT_SECRET").unwrap(),
    )
    .username(&env::var("REDDIT_USER_USERNAME").unwrap())
    .password(&env::var("REDDIT_USER_PASSWORD").unwrap())
    .login()
    .await
}
