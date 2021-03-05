extern crate clap;
#[macro_use]
extern crate prettytable;

mod enums;
pub mod output_producers;
mod types;

use clap::{App, Arg};
use enums::{Command, HTTPMethods};
use futures::future::join_all;
use output_producers::table_producer;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use types::{COUNTERMAP, METHOD, PAYLOAD, URL};

async fn counting_machine(counter_map: COUNTERMAP, mut rx: tokio::sync::mpsc::Receiver<Command>) {
    let mut map = counter_map.lock().await;
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Increment(code) => {
                if map.contains_key(&code) {
                    let value = map.get(&code).unwrap();
                    *map.get_mut(&code).unwrap() = value + 1;
                } else {
                    map.insert(code, 1);
                }
                // map.entry(100).or_insert(&0) += 1;
            }
            Command::Exit => {
                return;
            }
        };
    }
}

async fn send(
    method: METHOD,
    url: URL,
    payload: Option<PAYLOAD>,
    sender: tokio::sync::mpsc::Sender<Command>,
) {
    // Commond vars
    let target_url = url.clone();
    let client = reqwest::Client::new();
    let result; // For storing request result
    let method = method.clone();
    match &*method {
        HTTPMethods::GET => {
            if payload.is_some() {
                let content = payload.clone().unwrap();
                result = client.get(&*target_url).json(&*content).send().await;
            } else {
                result = client.get(&*target_url).send().await;
            }
        }
        HTTPMethods::POST => {
            if payload.is_some() {
                let content = payload.clone().unwrap();
                result = client.post(&*target_url).json(&*content).send().await;
            } else {
                result = client.post(&*target_url).send().await;
            }
        }
        HTTPMethods::PUT => {
            if payload.is_some() {
                let content = payload.clone().unwrap();
                result = client.put(&*target_url).json(&*content).send().await;
            } else {
                result = client.put(&*target_url).send().await;
            }
        }
        HTTPMethods::DELETE => {
            if payload.is_some() {
                let content = payload.clone().unwrap();
                result = client.delete(&*target_url).json(&*content).send().await;
            } else {
                result = client.delete(&*target_url).send().await;
            }
        }
        HTTPMethods::PATCH => {
            if payload.is_some() {
                let content = payload.clone().unwrap();
                result = client.patch(&*target_url).json(&*content).send().await;
            } else {
                result = client.patch(&*target_url).send().await;
            }
        }
    };
    match result {
        Ok(r) => {
            let command = Command::Increment(r.status().as_u16());
            sender.send(command).await.unwrap();
        }
        Err(_e) => {
            let command = Command::Increment(0);
            sender.send(command).await.unwrap();
        }
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
                .takes_value(true)
                .default_value(""),
        )
        .arg(Arg::with_name("method")
                .short("m")
                .long("method")
                .value_name("method")
                .help("Type of request to sent")
                .takes_value(true)
                .possible_values(&["GET", "POST", "PUT", "PATCH", "DELETE"])
                .default_value("GET")
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
    let method = matches.value_of("method").unwrap().to_string();
    let payload_filename = matches.value_of("payload").unwrap().to_string();
    let total_requests: i32 = matches.value_of("requests").unwrap().parse().unwrap();

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
                        println!("ERROR: {}", e);
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
        payload = Some(Arc::new(content.unwrap()));
    }

    // Variables shared between tasks
    let shared_url = Arc::new(url.to_string());
    let shared_method = Arc::new(HTTPMethods::fromstr(method.to_string()).unwrap());
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
        let shared_method = shared_method.clone();
        handles.push(tokio::spawn(async move {
            send(shared_method, shared_url, payload, sender).await
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
    sender.send(Command::Exit).await.unwrap();
    let _ = counting_machine_handle.await;

    let c = counter.clone();
    table_producer::produce_tabular_output(c).await;
}
