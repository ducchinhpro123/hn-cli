use colored::Colorize;
use futures::future::join_all;
use rayon::slice::ParallelSliceMut;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Story {
    title: String,
    url: Option<String>,
    score: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // New, Top and Best Stories
    const API: &str = "https://hacker-news.firebaseio.com/v0/topstories.json";

    let response: Value = reqwest::get(API).await?.json().await?;
    let json_arr = response.as_array().unwrap();

    let futures = json_arr.iter().map(|id| async move {
        let story_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
        reqwest::get(story_url).await?.json::<Story>().await
    });

    let results = join_all(futures).await;
    let mut top_stories = Vec::new();

    for story in results.iter() {
        match story {
            Ok(story) => {
                top_stories.push(story);
            }
            Err(err) => eprintln!("Error fetching story: {}", err),
        }
    }

    top_stories.par_sort_by(|a, b| a.score.cmp(&b.score));

    for story in top_stories.iter() {
        let url_display = match &story.url {
            Some(url) => url.cyan().to_string(),
            None => "Link unavailable".red().to_string(),
        };
        println!(
            "{}: {} ({})",
            story.score.to_string().yellow(),
            story.title.white().bold(),
            url_display
        );
    }

    Ok(())
}
