use reqwest::blocking;
use scraper::{Html, Selector};
use std::collections::HashSet;

fn crawl_layer(
    selector: &Selector,
    current_layer: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> HashSet<String> {
    let mut next_layer = HashSet::new();
    if current_layer.is_empty() {
        current_layer.insert("https://www.rust-lang.org".to_string());
    }
    for url in current_layer.iter() {
        if url.ends_with(".exe")
            || url.ends_with(".tar.gz")
            || url.ends_with(".tar.xz")
            || url.ends_with(".sh")
            || url.ends_with(".msi")
            || url.ends_with(".zip")
            || url.ends_with(".pkg")
        {
            println!("Skipping binary asset {url}");
            continue;
        }
        println!("Crawling {}", url);
        let response = match blocking::get(url) {
            Ok(resp) => resp,
            Err(err) => {
                eprintln!("Error crawling {}: {}", err, url);
                continue;
            }
        };

        let html = match response.text() {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Error decoding the response: {}", err);
                continue;
            }
        };

        let document = Html::parse_document(&html);
        for link in document.select(selector) {
            let href = link.value().attr("href");
            if let Some(url) = href {
                let new_url = format!("https://www.rust-lang.org{url}");
                // These visited.insert statements check if the url has already been visited
                // because visited.insert returns a boolean depending on if the url has already been
                // added to the visited hashmap
                if url.starts_with("/") && visited.insert(new_url.clone()) {
                    next_layer.insert(new_url);
                } else if url.starts_with("http")
                    && url.contains("rust-lang.org")
                    && visited.insert(new_url.clone())
                {
                    next_layer.insert(url.to_string());
                }
            }
        }
    }
    next_layer
}
fn main() {
    let selector = Selector::parse("a").unwrap();

    let mut visited: HashSet<String> = HashSet::new();
    let mut current_layer = HashSet::new();
    let mut next_layer = HashSet::new();

    for i in 1..4 {
        if i == 1 {
            println!("Starting layer 1");
            next_layer = crawl_layer(&selector, &mut current_layer, &mut visited)
        } else {
            // The first layer doesn't need to extend the visited and current_layer HashSets
            println!("Starting layer {i}");
            visited.extend(current_layer.drain());
            current_layer.extend(next_layer.drain());
            next_layer = crawl_layer(&selector, &mut current_layer, &mut visited);
        }
    }
    for link in visited.iter() {
        println!("{link}")
    }
}
