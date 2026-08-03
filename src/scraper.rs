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
    // let response = client
    //     .get(format!("https://x.com/{}", username))
    //     .header("User-Agent", "Mozilla/5.0") // X.com blocks requests without a User-Agent
    //     .send()
    //     .await;

    // if response.is_err() {
    //     red(format!("Failed to get response for {}", username));
    //     return zero(username);
    // }
    // let response = response.unwrap();
    // let body = response.text().await;
    let body = std::fs::read_to_string("app.html");

    if body.is_err() {
        red(format!("Failed to get body for {}", username));
        return zero(username);
    }
    let body = body.unwrap();
    let document = Html::parse_document(&*body);
    let stat_selector = Selector::parse("div[itemtype='https://schema.org/InteractionCounter']");
    if stat_selector.is_err() {
        red(format!("Failed to parse selector for {}", username));
        return zero(username);
    }
    let stat_selector = stat_selector.unwrap();

    let name_selector = Selector::parse("meta[itemprop='name']").unwrap();
    let count_selector = Selector::parse("meta[itemprop='userInteractionCount']").unwrap();

    let mut follows = 0u64;
    let mut friends = 0u64;
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

fn zero(username: String) -> CrawlResult {
    return CrawlResult::new(username, 0, 0, 0, true);
}

fn red(text: String) {
    println!("\x1b[31m {} \x1b[0m", text);
}
