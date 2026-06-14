use crate::utils::output::CrawlResult;
use crate::utils::parse_count;
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::path::PathBuf;
use std::process::exit;

pub async fn get_data(
    executable_path: &PathBuf,
    delay: u64,
    usernames: Vec<String>,
) -> Vec<CrawlResult> {
    println!("Starting browser");
    let config = BrowserConfig::builder()
        .chrome_executable(executable_path)
        .with_head()
        .build()
        .expect("Failed to build browser config");
    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .expect("Failed to lauch browser");
    // spawn a new task that continuously polls the handler
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });
    println!("Browser launched");
    let page = browser
        .new_page("https://example.com")
        .await
        .expect("Failed to create page");
    let resulr = collect(usernames, page, delay).await;
    return resulr;
}
async fn collect(usernames: Vec<String>, page: Page, delay: u64) -> Vec<CrawlResult> {
    let mut r = Vec::<CrawlResult>::new();
    for u in usernames {
        println!("Going to page for {}", u);
        let url = format!("https://x.com/{}", u);
        let new_page = page.goto(url.clone()).await;
        if new_page.is_err() {
            println!("Failed to goto page {}", url);
            exit(1);
        }
        let page = new_page.unwrap();
        let following = page
            .find_element(format!("a[href='/{}/following'] div div", u))
            .await;
        if following.is_err() {
            println!("Failed to find following count for {}, skipping", u);
            continue;
        }
        let followers = page
            .find_element(format!("a[href='/{}/verified_followers'] div div", u))
            .await;
        if followers.is_err() {
            println!("Failed to find followers count for {}, skipping", u);
            continue;
        }
        let following = following.unwrap().inner_text().await.unwrap().unwrap();
        let follower = followers.unwrap().inner_text().await.unwrap().unwrap();
        r.push(CrawlResult::new(
            u,
            parse_count(&*follower),
            parse_count(&*following),
        ));

        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    }

    r
}
