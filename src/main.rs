extern crate clap;
use clap::{App, Arg};
use futures::future::join_all;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::process::exit;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
// use std::vec::*;

type URL = Arc<Mutex<String>>;
type PAYLOAD = Arc<Mutex<Value>>;
type COUNTER_MAP = Arc<Mutex<HashMap<i32, AtomicI32>>>;

#[derive(Debug)]
enum Command {
    Increment(i32),
    Exit,
}

async fn counting_machine(counter_map: COUNTER_MAP, mut rx: tokio::sync::mpsc::Receiver<Command>) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Increment(code) => {
                let mut map = counter_map.lock().unwrap();
                if map.contains_key(&code) {
                    let status_code_counter = map.get_mut(&code).unwrap();
                    let value = status_code_counter.load(Ordering::Relaxed);
                    // status_code_counter.store(value + 1, Ordering::Relaxed);
                    *map.get_mut(&code).unwrap() = AtomicI32::new(value + 1);
                } else {
                    // println!("Hua");
                    map.insert(code, AtomicI32::new(1));
                }
                // map.entry(100).or_insert(&0) += 1;
                println!("Got increment")
            }
            Command::Exit => {
                println!("Exit ##############");
                return;
            }
        };
    }
}

async fn send(url: URL, payload: Option<PAYLOAD>, sender: tokio::sync::mpsc::Sender<Command>) {
    let target_url = url.lock().unwrap().clone();
    let client = reqwest::Client::new();
    let _result; // For storing request result
    if payload.is_some() {
        let content = payload.unwrap().lock().unwrap().clone();
        _result = client.post(&target_url).json(&content).send().await;
    } else {
        _result = client.post(&target_url).send().await;
    }
    let command = Command::Increment(100);
    sender.send(command).await.unwrap();
    /*match result {
        Ok(_) => println!(""),
        Err(e) => println!(""),
    }*/
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
        .arg (
            Arg::with_name("requests")
                .short("n")
                .long("requests")
                .default_value("0")
                .value_name("total_requests")
                    .help("Number of requests to send. Supply 0 or avoid supplying to send infinite number of requests")
        )
        .get_matches();

    // Extract user supplied values
    let url = matches.value_of("url").unwrap();
    let payload_filename = matches.value_of("payload").unwrap_or("").to_string();
    let total_requests: i32 = matches.value_of("requests").unwrap_or("0").parse().unwrap();

    let mut payload: Option<PAYLOAD> = None; // Sharable data
    let mut content: Option<Value> = None; // Stores JSON data

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
    let counter = Arc::new(Mutex::new(HashMap::new())); // Map of Atomic counters to keep HTTP status code count
    let (sender, receiver) = mpsc::channel(50);
    let counter_clone = counter.clone();
    let counting_machine_handle =
        tokio::spawn(async move { counting_machine(counter_clone, receiver) }.await);
    let mut index: i32 = 1;
    let mut handles = vec![];
    loop {
        let shared_url = shared_url.clone();
        let payload = payload.clone();
        let sender = sender.clone();
        handles.push(tokio::spawn(async move {
            send(shared_url, payload, sender).await
        }));
        if total_requests != 0 {
            if index == total_requests {
                break;
            } else {
                index += 1;
            }
        }
    }

    join_all(handles).await;
    println!("Sending Exit");
    sender.send(Command::Exit).await.unwrap();
    let _ = counting_machine_handle.await;
    println!("URL: {:?}", shared_url);
    let c = counter.clone();
    let c_clone = c.lock().unwrap();
    /*for key in &*c_clone {
        println!("{:?}", key);
    }*/
    println!("Map: {:?}", &*c_clone);
}
