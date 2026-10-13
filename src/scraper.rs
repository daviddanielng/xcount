use std::{fs, process::exit};

use crate::utils::enity::ProfileData;
use crate::utils::output::CrawlResult;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
struct ProfileSchema {
    #[serde(rename = "mainEntity")]
    main_entity: MainEntity,
}

#[derive(Debug, Deserialize)]
struct MainEntity {
    #[serde(rename = "interactionStatistic")]
    interaction_statistic: Vec<InteractionCounter>,
}

#[derive(Debug, Deserialize)]
struct InteractionCounter {
    name: String,
    #[serde(rename = "userInteractionCount")]
    user_interaction_count: u64,
}

pub async fn get_data(delay: u64, usernames: &Vec<String>) -> Vec<CrawlResult> {
    let mut data = Vec::new();
    let client = reqwest::Client::new();
    let mut proccessed = 0;
    for username in usernames {
        println!(
            "Estimated time remaining is {} minutes",
            (((usernames.len() - proccessed) as u64 * (delay + 1)) / 60)
        );

        data.push(collect(&client, username.clone()).await);
        proccessed = proccessed + 1;
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    }
    data
}

// X's profile page no longer emits JSON-LD (`div[itemtype='https://schema.org/...']`
// / `meta[itemprop=...]` are both gone from the DOM). Stats now live inside an inline
// <script> that hydrates a Relay store via `$R[n]=...` assignments — not valid JSON
// (unquoted keys, `!0`/`!1` booleans, `$R[n]` backreferences), so we can't select or
// parse it structurally. Instead we anchor on the one flat, alphabetically-keyed
// object that carries followers/following/tweets next to screenName, e.g.:
//   followers:14225857,following:1381,...,restId:"32660559",screenName:"wizkidayo",tweets:54325
static PROFILE_STATS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)followers:(\d+),following:(\d+),.*?restId:"(\d+)",.*?screenName:"([^"]+)",tweets:(\d+)"#,
    )
    .expect("static regex is valid")
});

async fn use_relay_script(body: String, username: String) -> CrawlResult {
    let Some(caps) = PROFILE_STATS_RE.captures(&body) else {
        red(format!("Failed to find profile stats for {}", username));
        return zero(username);
    };

    // Guard against matching some other user's block embedded in the page
    // (e.g. a reply author or quoted tweet's profile data).
    let matched_screen_name = &caps[4];
    if !matched_screen_name.eq_ignore_ascii_case(&username) {
        red(format!(
            "Profile stats screen_name mismatch for {} (got {})",
            username, matched_screen_name
        ));
        return zero(username);
    }

    let follows: u64 = match caps[1].parse() {
        Ok(v) => v,
        Err(_) => {
            red(format!("Failed to parse follower count for {}", username));
            return zero(username);
        }
    };
    let friends: u64 = match caps[2].parse() {
        // "Following"
        Ok(v) => v,
        Err(_) => {
            red(format!("Failed to parse following count for {}", username));
            return zero(username);
        }
    };
    let tweets: u64 = match caps[5].parse() {
        Ok(v) => v,
        Err(_) => {
            red(format!("Failed to parse tweet count for {}", username));
            return zero(username);
        }
    };

    CrawlResult::new(username, follows, friends, tweets, false)
}

fn use_initial_state(body: &str, username: &str) -> Option<CrawlResult> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("script").ok()?;

    for script in document.select(&selector) {
        let text: String = script.text().collect();
        let Some((_, state_text)) = text.split_once("window.__INITIAL_STATE__") else {
            continue;
        };
        let state_text = state_text.trim_start();
        let Some(state_text) = state_text.strip_prefix('=') else {
            continue;
        };
        let state_text = state_text.trim_start();
        let state_text = state_text.strip_suffix(';').unwrap_or(state_text).trim();
        let Ok(state) = serde_json::from_str::<serde_json::Value>(state_text) else {
            continue;
        };
        let Some(users) = state
            .pointer("/entities/users/entities")
            .and_then(serde_json::Value::as_object)
        else {
            continue;
        };

        let Some(user) = users.values().find(|user| {
            user.get("screen_name")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|screen_name| screen_name.eq_ignore_ascii_case(username))
        }) else {
            continue;
        };

        let followers = user.get("followers_count")?.as_u64()?;
        let following = user.get("friends_count")?.as_u64()?;
        let tweets = user.get("statuses_count")?.as_u64()?;

        return Some(CrawlResult::new(
            username.to_owned(),
            followers,
            following,
            tweets,
            false,
        ));
    }

    None
}

pub async fn use_app_script(body: String, username: String) -> CrawlResult {
    let document = Html::parse_document(&*body);
    let script_selector = match Selector::parse(r#"script[data-testid="UserProfileSchema-test"]"#) {
        Ok(s) => s,
        Err(_) => {
            red(format!("Failed to parse selector for {}", username));
            return zero(username);
        }
    };

    let Some(script_el) = document.select(&script_selector).next() else {
        red(format!("No profile schema found for {}", username));
        return zero(username);
    };

    let json_text: String = script_el.text().collect();

    let schema: ProfileSchema = match serde_json::from_str(&json_text) {
        Ok(s) => s,
        Err(e) => {
            red(format!("Failed to parse JSON-LD for {}: {}", username, e));
            return zero(username);
        }
    };
    let mut follows = 0u64;
    let mut friends = 0u64; // "Following"
    let mut tweets = 0u64;

    for stat in schema.main_entity.interaction_statistic {
        match stat.name.as_str() {
            "Follows" => follows = stat.user_interaction_count,
            "Friends" => friends = stat.user_interaction_count,
            "Tweets" => tweets = stat.user_interaction_count,
            _ => {}
        }
    }
    return CrawlResult::new(username, follows, friends, tweets, false);
}
async fn use_ld_json(body: String, username: String) -> CrawlResult {
    let document = Html::parse_document(&body);
    let selector = match Selector::parse(r#"script[type="application/ld+json"]"#) {
        Ok(selector) => selector,
        Err(_) => {
            red(format!("Failed to parse JSON-LD selector for {}", username));
            return zero(username);
        }
    };

    let Some(script) = document.select(&selector).next() else {
        red(format!("No JSON-LD profile schema found for {}", username));
        return zero(username);
    };

    let json_text: String = script.text().collect();
    let profile: ProfileData = match serde_json::from_str(&json_text) {
        Ok(profile) => profile,
        Err(error) => {
            red(format!(
                "Failed to parse JSON-LD for {}: {}",
                username, error
            ));
            return zero(username);
        }
    };

    let mut follows = 0;
    let mut friends = 0;
    let mut tweets = 0;
    let mut found_stat = false;

    for stat in profile.main_entity.interaction_statistic {
        match stat.name.as_str() {
            "Follows" => {
                follows = stat.count;
                found_stat = true;
            }
            "Friends" => {
                friends = stat.count;
                found_stat = true;
            }
            "Tweets" => {
                tweets = stat.count;
                found_stat = true;
            }
            _ => {}
        }
    }

    if !found_stat {
        red(format!("Interaction stats not found for {}.", username));
        return zero(username);
    }

    CrawlResult::new(username, follows, friends, tweets, false)
}

async fn use_profile_data(body: String, username: String) -> CrawlResult {
    let has_json_ld = Selector::parse(r#"script[type="application/ld+json"]"#)
        .ok()
        .is_some_and(|selector| {
            Html::parse_document(&body)
                .select(&selector)
                .next()
                .is_some()
        });

    if has_json_ld {
        use_ld_json(body, username).await
    } else if let Some(result) = use_initial_state(&body, &username) {
        result
    } else {
        use_relay_script(body, username).await
    }
}

async fn use_div(body: String, username: String) -> CrawlResult {
    let document = Html::parse_document(&*body);
    let stat_selector = Selector::parse(
        "div[itemprop='mainEntity'] > div[itemtype='https://schema.org/InteractionCounter']",
    );
    if stat_selector.is_err() {
        red(format!("Failed to parse selector for {}", username));
        return zero(username);
    }
    let stat_selector = stat_selector.unwrap();

    let name_selector = Selector::parse("meta[itemprop='name']").unwrap();
    let count_selector = Selector::parse("meta[itemprop='userInteractionCount']").unwrap();

    let mut follows = 0u64;
    let mut friends = 0u64; // "Following"
    let mut tweets = 0u64;
    let mut found_any = false;

    for stat_div in document.select(&stat_selector) {
        let name = stat_div
            .select(&name_selector)
            .next()
            .and_then(|el| el.value().attr("content"));

        let count = stat_div
            .select(&count_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .and_then(|c| c.parse::<u64>().ok());

        if let (Some(name), Some(count)) = (name, count) {
            found_any = true;
            match name {
                "Follows" => follows = count,
                "Following" => friends = count,
                "Tweets" => tweets = count,
                _ => {}
            }
        }
    }

    if !found_any {
        red(format!("Interaction stats not found for {}.", username));
        return zero(username);
    }

    return CrawlResult::new(username, follows, friends, tweets, false);
}
pub async fn collect(client: &reqwest::Client, username: String) -> CrawlResult {
    println!("Getting username for {}", username);
    let response = client
        .get(format!("https://x.com/{}", username))
.header(
    "User-Agent",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
)        .send()
        .await;

    if response.is_err() {
        red(format!("Failed to get response for {}", username));
        return zero(username);
    }
    let response = response.unwrap();
    let body = response.text().await;

    if body.is_err() {
        red(format!("Failed to get body for {}", username));
        return zero(username);
    }
    let body = body.unwrap();
    std::fs::write("./hello.html", &body).expect("TODO: panic message");

    let resultt = use_profile_data(body, username).await;
    println!("{:?}", resultt);
    return resultt;
}

fn zero(username: String) -> CrawlResult {
    return CrawlResult::new(username, 0, 0, 0, true);
}

fn red(text: String) {
    println!("\x1b[31m {} \x1b[0m", text);
}

#[cfg(test)]
mod tests {
    use super::{use_initial_state, use_relay_script};

    #[tokio::test]
    async fn relay_script_reads_current_profile_shape() {
        let body = r#"<script>($R=>$R[0]={profile:{followers:180,following:463,hasHighlights:!1,isUnavailable:!1,isVerified:!1,location:"wherever",name:"Daniel",possiblySensitive:!1,profileStatus:"Available",restId:"972624651993141248",schema:null,screenName:"daviddanielng",tweets:6852}})($R);</script>"#;
        let result = use_relay_script(body.to_owned(), "daviddanielng".to_owned()).await;

        assert_eq!(result.followers, 180);
        assert_eq!(result.following, 463);
        assert_eq!(result.tweets, 6852);
        assert!(!result.error);
    }

    #[test]
    fn initial_state_selects_counts_for_requested_screen_name() {
        let body = r#"<script>window.__INITIAL_STATE__
            = {"entities":{"users":{"entities":{"1":{"screen_name":"other","followers_count":1,"friends_count":2,"statuses_count":3},"2":{"screen_name":"daviddanielng","followers_count":180,"friends_count":463,"statuses_count":6852}}}}};</script>"#;
        let result = use_initial_state(body, "daviddanielng").unwrap();

        assert_eq!(result.followers, 180);
        assert_eq!(result.following, 463);
        assert_eq!(result.tweets, 6852);
        assert!(!result.error);
    }
}
