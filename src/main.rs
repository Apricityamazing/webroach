use scraper::{Html, Selector};
use std::collections::HashSet;

enum LinkResult {
    Http {
        url: reqwest::Url,
        status: reqwest::StatusCode,
    },
    NetworkError {
        url: String,
        err_message: String,
    },
}
fn crawl_layer(
    client: &reqwest::blocking::Client,
    selector: &Selector,
    current_layer: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    external_links: &mut HashSet<String>,
) -> HashSet<String> {
    let mut next_layer = HashSet::new();
    if current_layer.is_empty() {
        current_layer.insert("https://www.rust-lang.org".to_string());
    }
    for url in current_layer.drain() {
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
        let response = match client.get(&url).send() {
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
                if url.starts_with("/") {
                    // Relative internal link
                    if visited.insert(new_url.clone()) {
                        next_layer.insert(new_url);
                    }
                } else if url.starts_with("http") && url.contains("rust-lang.org") {
                    // Absolute internal link
                    if visited.insert(url.to_string()) {
                        next_layer.insert(url.to_string());
                    }
                } else if url.starts_with("http") && external_links.insert(url.to_string()) {
                    println!("Added {} to external links", url);
                }
            }
        }
    }
    next_layer
}

fn check_links(
    client: &reqwest::blocking::Client,
    external_links: &mut HashSet<String>,
) -> Vec<LinkResult> {
    let mut checked_links: Vec<LinkResult> = Vec::new();
    for url in external_links.drain() {
        println!("Checking external url {}", url);
        match client.head(&url).send() {
            Ok(resp) => {
                checked_links.push(LinkResult::Http {
                    url: (resp.url().clone()),
                    status: (resp.status()),
                });
            }
            Err(err) => {
                checked_links.push(LinkResult::NetworkError {
                    url: (url.clone()),
                    err_message: (err.to_string()),
                });
            }
        };
    }

    checked_links
}
fn main() {
    let client = reqwest::blocking::Client::new();
    let selector = Selector::parse("a").unwrap();

    let mut visited: HashSet<String> = HashSet::new();
    let mut current_layer = HashSet::new();
    let mut external_links: HashSet<String> = HashSet::new();
    let mut next_layer = HashSet::new();

    for i in 1..4 {
        if i == 1 {
            println!("Starting layer 1");
            next_layer = crawl_layer(
                &client,
                &selector,
                &mut current_layer,
                &mut visited,
                &mut external_links,
            )
        } else {
            // The first layer doesn't need to extend the visited and current_layer HashSets
            println!("Starting layer {i}");
            visited.extend(current_layer.drain());
            current_layer.extend(next_layer.drain());
            next_layer = crawl_layer(
                &client,
                &selector,
                &mut current_layer,
                &mut visited,
                &mut external_links,
            );
        }
    }

    let checked_links = check_links(&client, &mut external_links);

    for link in checked_links.iter() {
        match link {
            LinkResult::Http { url, status } => {
                println!("Url: {} Returned Status: {}", url, status);
            }
            LinkResult::NetworkError { url, err_message } => {
                println!("Url: {} Network Error Status {}", url, err_message)
            }
        };
    }

    for link in visited.iter() {
        println!("{link}")
    }
}
