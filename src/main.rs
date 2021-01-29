extern crate clap;
use clap::{App, Arg};
use serde_json::Value;
use std::fs;
use std::process::exit;
use std::sync::{Arc, Mutex};

type URL = Arc<Mutex<String>>;
type PAYLOAD = Arc<Mutex<Value>>;

async fn send(url: URL, payload: Option<PAYLOAD>) {
    let target_url = url.lock().unwrap().clone();
    let client = reqwest::Client::new();
    let result; // For storing request result
    if payload.is_some() {
        let content = payload.unwrap().lock().unwrap().clone();
        result = client.post(&target_url).json(&content).send().await;
    } else {
        result = client.post(&target_url).send().await;
    }
    match result {
        Ok(_) => println!("Found something"),
        Err(e) => println!("{:?}", e),
    }
}

#[tokio::main]
async fn main() {
    // Prepare for argument parsing
    let matches = App::new("stresster")
        .version("1.0")
        .author("Malhar Vora <vbmade2000@gmail.com>")
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("url")
                .help("API URL to hit")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("payload")
                .short("p")
                .long("payload")
                .value_name("payload")
                .help("File containing payload to send")
                .takes_value(true),
        )
        .get_matches();

    let url = matches.value_of("url").unwrap();
    let payload_filename = matches.value_of("payload").unwrap_or("").to_string();
    let mut payload: Option<PAYLOAD> = None;
    let mut content: Option<Value> = None;

    // Process payload only if filename supplied
    if !payload_filename.is_empty() {
        let result = fs::read_to_string(payload_filename.to_string());
        match result {
            Ok(r) => {
                let result = serde_json::from_str(&r);
                match result {
                    Ok(p) => content = Some(p),
                    Err(e) => {
                        println!("ERROR: Invalid JSON");
                        println!("{}", e);
                        exit(1)
                    }
                }
            }
            Err(e) => {
                println!("ERROR: {}, {}", payload_filename, e);
                exit(1)
            }
        }
    };

    // We share payload only if we have something to share
    if content.is_some() {
        payload = Some(Arc::new(Mutex::new(content.unwrap())));
    }

    let shared_url = Arc::new(Mutex::new(url.to_string()));

    loop {
        let shared_url = shared_url.clone();
        let payload = payload.clone();
        tokio::spawn(async move { send(shared_url, payload).await });
    }
}
