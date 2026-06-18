use crate::utils::enity::ProfileData;
use crate::utils::output::CrawlResult;
use scraper::{Html, Selector};
use std::process::exit;

pub async fn get_data(delay: u64, usernames: &Vec<String>) -> Vec<CrawlResult> {
    let mut data = Vec::new();
    let client = reqwest::Client::new();
    for username in usernames {
        let g = collect(&client, username.clone()).await;
        if g.is_none() {
            println!("Failed to get data for {}", username);
            continue;
        }
        let g = g.unwrap();
        data.push(g);
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    }
    data
}

pub async fn collect(client: &reqwest::Client, username: String) -> Option<CrawlResult> {
    println!("Getting username for {}", username);
    let response = client
        .get(format!("https://x.com/{}", username))
        .header("User-Agent", "Mozilla/5.0") // X.com blocks requests without a User-Agent
        .send()
        .await;

    if response.is_err() {
        println!("Failed to get response for {}", username);
        return None;
    }
    let response = response.unwrap();
    let body = response.text().await;

    if body.is_err() {
        println!("Failed to get body for {}", username);
        return None;
    }
    let body = body.unwrap();
    let document = Html::parse_document(&*body);
    let selector = Selector::parse("script[type='application/ld+json']");
    if selector.is_err() {
        println!("Failed to parse selector for {}", username);
        return None;
    }
    let selector = selector.unwrap();
    if let Some(element) = document.select(&selector).nth(1) {
        let json_text: String = element.text().collect();

        let profile = serde_json::from_str(&*json_text);
        if profile.is_err() {
            println!("Failed to parse JSON-LD for {}", username);
            return None;
        }
        let profile: ProfileData = profile.unwrap();
        // Extract the specific counts from the array
        let mut follows = 0;
        let mut friends = 0;
        let mut tweets = 0;
        for stat in profile.main_entity.interaction_statistic {
            match stat.name.as_str() {
                "Follows" => follows = stat.count,
                "Friends" => friends = stat.count,
                "Tweets" => tweets = stat.count,
                _ => {}
            }
        }
        return Some(CrawlResult::new(username, follows, friends, tweets));
    } else {
        println!("Json not found for {}.", username);
    }

    None
}
