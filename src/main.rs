use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
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
    selector: Selector,
    current_layer: &mut HashSet<String>,
    visited: Arc<Mutex<HashSet<String>>>,
    external_links: Arc<Mutex<HashSet<String>>>,
) -> HashSet<String> {
    let next_layer = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = vec![];
    if current_layer.is_empty() {
        current_layer.insert("https://www.rust-lang.org".to_string());
    }
    for url in current_layer.drain() {
        let selector_clone = selector.clone();
        let client_clone = client.clone();
        let next_layer_handle = next_layer.clone();
        let visited_handle = visited.clone();
        let external_links_handle = external_links.clone();
        let handle = thread::spawn(move || {
            if url.ends_with(".exe")
                || url.ends_with(".tar.gz")
                || url.ends_with(".tar.xz")
                || url.ends_with(".sh")
                || url.ends_with(".msi")
                || url.ends_with(".zip")
                || url.ends_with(".pkg")
            {
                println!("Skipping binary asset {url}");
                return;
            }
            println!("Crawling {}", url);
            let response = match client_clone.get(&url).send() {
                Ok(resp) => resp,
                Err(err) => {
                    eprintln!("Error crawling {}: {}", err, url);
                    return;
                }
            };

            let html = match response.text() {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("Error decoding the response: {}", err);
                    return;
                }
            };

            let document = Html::parse_document(&html);
            for link in document.select(&selector_clone) {
                let href = link.value().attr("href");
                if let Some(url) = href {
                    let new_url = format!("https://www.rust-lang.org{url}");
                    if url.starts_with("/") {
                        // Relative internal link
                        if visited_handle.lock().unwrap().insert(new_url.clone()) {
                            next_layer_handle.lock().unwrap().insert(new_url);
                        }
                    } else if url.starts_with("http") && url.contains("rust-lang.org") {
                        // Absolute internal link
                        if visited_handle.lock().unwrap().contains(url) {
                            next_layer_handle.lock().unwrap().insert(url.to_string());
                        }
                    } else if url.starts_with("http")
                        && external_links_handle
                            .lock()
                            .unwrap()
                            .insert(url.to_string())
                    {
                        println!("Added {} to external links", url);
                    }
                }
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    next_layer.lock().unwrap().clone()
}

fn check_links(
    client: &reqwest::blocking::Client,
    external_links: Arc<Mutex<HashSet<String>>>,
) -> Vec<LinkResult> {
    let mut handles = vec![];
    let checked_links = Arc::new(Mutex::new(Vec::new()));
    for url in external_links.lock().unwrap().drain() {
        let checked_links_clone = checked_links.clone();
        let client_clone = client.clone();
        let handle = thread::spawn(move || {
            match client_clone.head(&url).send() {
                Ok(resp) => {
                    checked_links_clone.lock().unwrap().push(LinkResult::Http {
                        url: (resp.url().clone()),
                        status: (resp.status()),
                    });
                }
                Err(err) => {
                    checked_links_clone
                        .lock()
                        .unwrap()
                        .push(LinkResult::NetworkError {
                            url: (url.clone()),
                            err_message: (err.to_string()),
                        });
                }
            };
            println!("Checked external url {}", url);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    checked_links.lock().unwrap().clone()
}
fn main() {
    let client = reqwest::blocking::Client::new();
    let selector = Selector::parse("a").unwrap();

    let visited = Arc::new(Mutex::new(HashSet::new()));
    let mut current_layer = HashSet::new();
    let external_links = Arc::new(Mutex::new(HashSet::new()));
    let mut next_layer = HashSet::new();

    for i in 1..4 {
        if i == 1 {
            println!("Starting layer 1");
            next_layer = crawl_layer(
                &client,
                selector.clone(),
                &mut current_layer,
                visited.clone(),
                external_links.clone(),
            )
        } else {
            // The first layer doesn't need to extend the visited and current_layer HashSets
            println!("Starting layer {i}");
            visited.lock().unwrap().extend(current_layer.drain());
            current_layer.extend(next_layer.drain());
            next_layer = crawl_layer(
                &client,
                selector.clone(),
                &mut current_layer,
                visited.clone(),
                external_links.clone(),
            );
        }
    }

    let checked_links = check_links(&client, external_links.clone());

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
}
