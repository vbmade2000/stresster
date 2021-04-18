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
<<<<<<< HEAD
mod tests;
=======
>>>>>>> Add integration test to check if log file is created after execution
mod types;

use clap::{App, Arg};
use data::Data;
use enums::{Command, HTTPMethods};
use futures::future::join_all;
use output_producers::{json_producer, table_producer};
use reqwest::{
    header::{HeaderName, HeaderValue},
    Client,
};
use serde_json::Value;
use slog::Drain;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use types::{COUNTERMAP, DATA, LOGGER};

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

async fn send(sender: tokio::sync::mpsc::Sender<Command>, logger: LOGGER, data: DATA) {
    // Common vars
    let result; // For storing request result
    let logger = logger.clone();

    let data = data.clone();
    let payload = &data.payload;
    let headers = data.headers.clone();
    let method = data.method.clone();
    let target_url = data.url.to_string();
    let ssl_cert = &data.cert_path;
    info!(
        logger,
        "Sending {:?} request to {:?} with payload {:?}", method, target_url, payload
    );

    // Build client with required configurations
    let mut client = Client::builder()
        .default_headers(headers.clone())
        .build()
        .unwrap();

    if ssl_cert.len() > 0 {
        let mut buf = Vec::new();
        fs::File::open(ssl_cert)
            .expect(format!("ERROR: Error reading {:?}", ssl_cert).as_str())
            .read_to_end(&mut buf)
            .expect("ERROR: Error reading file");
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        client = Client::builder()
            .add_root_certificate(cert)
            .default_headers(headers)
            .build()
            .unwrap();
    }
    match method {
        HTTPMethods::GET => {
            result = client.get(&*target_url).json(payload).send().await;
        }
        HTTPMethods::POST => {
            result = client.post(&*target_url).json(payload).send().await;
        }
        HTTPMethods::PUT => {
            result = client.put(&*target_url).json(payload).send().await;
        }
        HTTPMethods::DELETE => {
            result = client.delete(&*target_url).json(payload).send().await;
        }
        HTTPMethods::PATCH => {
            result = client.patch(&*target_url).json(payload).send().await;
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
        .version("0.1.0")
        .author("Malhar Vora <vbmade2000@gmail.com>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("config")
                .help("Configuration file containing data to send")
                .takes_value(true)
                .required(true),
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
    let output_format = matches.value_of("format").unwrap();
    let config_filename = matches.value_of("config").unwrap().to_string();
    let total_requests: i32 = matches.value_of("requests").unwrap().parse().unwrap();

    let shared_data: DATA; // Data shared between tasks
    let content: Value; // Stores JSON data read from file

    // TODO: Make error handling compact
    let result = fs::read_to_string(config_filename.to_string());
    match result {
        Ok(r) => {
            let result = serde_json::from_str(&r);
            match result {
                Ok(p) => content = p,
                Err(e) => {
                    println!("ERROR: {}", e);
                    exit(1)
                }
            }
        }
        Err(e) => {
            println!("ERROR: {}, {}", config_filename, e);
            exit(1)
        }
    };

    // Default values in case actual values are not supplied
    let default_payload: serde_json::Value = serde_json::from_str("{}").unwrap();
    let default_headers: serde_json::Value = serde_json::from_str("{}").unwrap();
    let default_path: serde_json::Value = serde_json::from_str("{}").unwrap();

    // Extract payload or get default payload
    let actual_payload = content.get("payload").unwrap_or(&default_payload);
    let mut data: Data = Data::default();
    data.payload = actual_payload.clone();

    // Extract HTTP headers or get default HTTP headers
    let headers = content
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

    // Extract HTTP method
    let method = content
        .get("method")
        .expect("ERROR: Please specify method in payload file")
        .as_str()
        .unwrap()
        .to_string();

    // Grab enum value from string HTTP method
    let http_method = HTTPMethods::fromstr(method.to_uppercase());
    if !http_method.is_some() {
        println!("ERROR: Invalid HTTP method {:?}", method);
        exit(1);
    }
    data.method = http_method.unwrap();

    // Extract URL
    data.url = content
        .get("url")
        .expect("ERROR: Please specify URL in payload file")
        .as_str()
        .unwrap()
        .to_string();

    // Extract cert_path if spplied
    data.cert_path = content
        .get("ssl_cert")
        .unwrap_or(&default_path)
        .as_str()
        .unwrap_or("")
        .to_string();

    shared_data = Arc::new(data);

    // Variables shared between tasks
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
        let sender = sender.clone();
        let logger = shared_logger.clone();
        let shared_data = shared_data.clone();
        handles.push(tokio::spawn(async move {
            send(sender, logger, shared_data).await
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
