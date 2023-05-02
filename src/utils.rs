use chrono::Local;

pub fn get_footer() -> String {
    let usage = "✍️ [Usage](https://github.com/aquelemiguel/itaots-reddit#usage)";
    let issues = "🐛 [Report issues](https://github.com/aquelemiguel/itaots-reddit/issues)";
    let github = "⭐️ [GitHub](https://github.com/aquelemiguel/itaots-reddit)";
    [usage, issues, github].join(" | ")
}

pub fn format_bulletpoint(pair: &(String, String), acronym: &str) -> String {
    let (artist, album) = pair;
    format!("💿 **{acronym}** is **{album}** by {artist}")
}

pub fn build_query(acronym: &str) -> String {
    acronym
        .chars()
        .filter_map(|c| (c.is_alphanumeric() || c.is_whitespace()).then_some(format!("{}* ", c)))
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn log(emoji: char, msg: &str) {
    let ts = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    println!("{ts} | {emoji} {msg}");
}
