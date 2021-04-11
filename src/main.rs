extern crate clap;
#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod data;
mod enums;
pub mod output_producers;
mod types;

use clap::{App, Arg};
use data::Data;
use enums::{Command, HTTPMethods};
use futures::future::join_all;
use output_producers::{json_producer, table_producer};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use serde_json::Value;
use slog::Drain;
use std::collections::HashMap;
use std::fs;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use types::{COUNTERMAP, DATA, LOGGER, METHOD, URL};

const LOG_PATH: &str = "stresster.log";

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
    sender: tokio::sync::mpsc::Sender<Command>,
    logger: LOGGER,
    data: Option<DATA>,
) {
    // Common vars
    let target_url = url.clone();
    let client = Client::new();
    let result; // For storing request result
    let method = method.clone();
    let logger = logger.clone();

    /* A default value for payload in case any of them is not supplied. We can put a condition
       to check if they are supplied but that doesn't feel idiomatic way.
    */
    let default_payload: serde_json::Value = serde_json::from_str("{}").unwrap();

    let mut a_payload = &default_payload;
    let mut a_headers = HeaderMap::new();
    let adata: std::sync::Arc<data::Data>;

    if data.is_some() {
        adata = data.clone().unwrap();
        a_payload = &adata.payload;
        a_headers = adata.headers.clone();
    }

    match &*method {
        HTTPMethods::GET => {
            info!(
                logger,
                "Sending {:?} request to {:?} with payload {:?}", method, target_url, a_payload
            );
            result = client
                .get(&*target_url)
                .json(a_payload)
                .headers(a_headers.clone())
                .send()
                .await;
        }
        HTTPMethods::POST => {
            info!(
                logger,
                "Sending {:?} request to {:?} with payload {:?}", method, target_url, a_payload
            );
            result = client
                .post(&*target_url)
                .json(a_payload)
                .headers(a_headers.clone())
                .send()
                .await;
        }
        HTTPMethods::PUT => {
            info!(
                logger,
                "Sending {:?} request to {:?} with payload {:?}", method, target_url, a_payload
            );
            result = client
                .put(&*target_url)
                .json(a_payload)
                .headers(a_headers.clone())
                .send()
                .await;
        }
        HTTPMethods::DELETE => {
            info!(
                logger,
                "Sending {:?} request to {:?} with payload {:?}", method, target_url, a_payload
            );
            result = client
                .delete(&*target_url)
                .json(a_payload)
                .headers(a_headers.clone())
                .send()
                .await;
        }
        HTTPMethods::PATCH => {
            info!(
                logger,
                "Sending {:?} request to {:?} with payload {:?}", method, target_url, a_payload
            );
            result = client
                .patch(&*target_url)
                .json(a_payload)
                .headers(a_headers.clone())
                .send()
                .await;
        }
    };
    match result {
        Ok(r) => {
            let command = Command::Increment(r.status().as_u16());
            info!(logger, "Result status code: {}", r.status().as_u16());
            sender.send(command).await.unwrap();
        }
        Err(e) => {
            let command = Command::Increment(0);
            error!(logger, "Result error : {}", e);
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
        .arg(Arg::with_name("format")
                .short("f")
                .long("format")
                .value_name("format")
                .help("Output format")
                .takes_value(true)
                .possible_values(&["table", "json"])
                .default_value("table")
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
    let output_format = matches.value_of("format").unwrap();
    let method = matches.value_of("method").unwrap().to_string();
    let payload_filename = matches.value_of("payload").unwrap().to_string();
    let total_requests: i32 = matches.value_of("requests").unwrap().parse().unwrap();

    let mut shared_data: Option<DATA> = None; // Data shared between tasks
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

    let default_payload: serde_json::Value = serde_json::from_str("{}").unwrap();
    let default_headers: serde_json::Value = serde_json::from_str("{}").unwrap();

    // We share payload only if we have something to share
    if content.is_some() {
        let unwrapped_content = content.unwrap();

        // Extract payload if supplied
        let actual_payload = unwrapped_content.get("payload").unwrap_or(&default_payload);
        let mut data: Data = Data::default();
        data.payload = actual_payload.clone();

        // Extract HTTP headers
        let headers = unwrapped_content
            .get("headers")
            .unwrap_or(&default_headers)
            .as_object()
            .unwrap();

        // Insert extracted headers to shared object
        for (key, value) in headers {
            data.headers.insert(
                HeaderName::from_lowercase(key.to_lowercase().as_bytes()).unwrap(),
                HeaderValue::from_str(value.as_str().unwrap()).unwrap(),
            );
        }

        shared_data = Some(Arc::new(data));
    }

    // Variables shared between tasks
    let shared_url = Arc::new(url.to_string());
    let shared_method = Arc::new(HTTPMethods::fromstr(method.to_string()).unwrap());
    let counter = Arc::new(Mutex::new(HashMap::new())); // Map of Atomic counters to keep HTTP status code count
    let (sender, receiver) = mpsc::channel(50);

    // Create a logger instance
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(LOG_PATH)
        .expect("ERROR: Unable to create a log file");
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_async::Async::new(slog_term::FullFormat::new(decorator).build().fuse())
        .build()
        .fuse();
    let logger = slog::Logger::root(drain, o!());
    let shared_logger = Arc::new(logger);

    // Start counter function
    let counter_clone = counter.clone();
    let counting_machine_handle =
        tokio::spawn(async move { counting_machine(counter_clone, receiver) }.await);

    let mut index: i32 = 1;
    let mut handles = vec![];
    loop {
        let shared_url = shared_url.clone();
        let sender = sender.clone();
        let shared_method = shared_method.clone();
        let logger = shared_logger.clone();
        let shared_data = shared_data.clone();
        handles.push(tokio::spawn(async move {
            send(shared_method, shared_url, sender, logger, shared_data).await
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
    let logger = shared_logger.clone();
    if output_format == "json" {
        json_producer::produce_json_output(c, logger).await;
    } else if output_format == "table" {
        table_producer::produce_tabular_output(c, logger).await;
    }
}
