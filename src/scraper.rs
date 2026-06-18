use crate::utils::enity::ProfileData;
use crate::utils::output::CrawlResult;
use scraper::{Html, Selector};

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

pub async fn collect(client: &reqwest::Client, username: String) -> CrawlResult {
    println!("Getting username for {}", username);
    let response = client
        .get(format!("https://x.com/{}", username))
        .header("User-Agent", "Mozilla/5.0") // X.com blocks requests without a User-Agent
        .send()
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
    let document = Html::parse_document(&*body);
    let selector = Selector::parse("script[type='application/ld+json']");
    if selector.is_err() {
        red(format!("Failed to parse selector for {}", username));
        return zero(username);
    }
    let selector = selector.unwrap();
    if let Some(element) = document.select(&selector).nth(1) {
        let json_text: String = element.text().collect();

        let profile = serde_json::from_str(&*json_text);
        if profile.is_err() {
            red(format!("Failed to parse JSON-LD for {}", username));
            return zero(username);
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
                _ => {
                    return zero(username);
                }
            }
        }
        return CrawlResult::new(username, follows, friends, tweets, false);
    } else {
        red(format!("Json not found for {}.", username));
    }

    return zero(username);
}

fn zero(username: String) -> CrawlResult {
    return CrawlResult::new(username, 0, 0, 0, true);
}

fn red(text: String) {
    println!("\x1b[31m {} \x1b[0m", text);
}
