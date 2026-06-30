use reqwest::blocking;
use scraper::{Html, Selector};
use std::collections::HashSet;

fn main() {
    let html = blocking::get("https://www.rust-lang.org")
        .unwrap()
        .text()
        .unwrap();
    let document = Html::parse_document(&html);
    let selector = Selector::parse("a").unwrap();

    let mut remote = HashSet::new();
    let mut local = HashSet::new();
    for element in document.select(&selector) {
        let href = element.value().attr("href");
        if let Some(url) = href {
            if url.starts_with("/") {
                local.insert(format!("https://www.rust-lang.org{url}"));
            } else if url.starts_with("http") {
                if url.contains("rust-lang.org") {
                    local.insert(url.to_string());
                } else {
                    remote.insert(url.to_string());
                }
            }
        }
    }
    println!("Amount of local links: {}", local.len());
    for link in local.iter() {
        println!("{link}");
    }
    println!("Amount of remote links: {}", remote.len());
    for link in remote.iter() {
        println!("{link}");
    }
}
