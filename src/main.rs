extern crate clap;
#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod enums;
mod helper;
pub mod output_producers;
mod request_data;
mod types;

use enums::{Command, HttpMethods};
use futures::future::join_all;
use helper::{extract_values_from_args, get_cmd_args, get_logger, get_request_data_from_file};
use output_producers::{json_producer, table_producer};
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use types::{Countermap, Data, Logger};

const LOG_PATH: &str = "stresster.log";

/// Counts number of requests for all the received status code based on data received from Send
/// function.
async fn counting_machine(counter_map: Countermap, mut rx: tokio::sync::mpsc::Receiver<Command>) {
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

/// Actual sends the{GET, POST, PUT, PATCH, DELETE} requests to URL configured in Data file.
/// Sends return code to couting_machine function for accouting.
async fn send(sender: tokio::sync::mpsc::Sender<Command>, logger: Logger, data: Data) {
    // Common vars
    let result; // For storing request result
    let logger = logger.clone();

    let data = data.clone();
    let payload = &data.payload;
    let headers = data.headers.clone();
    let method = data.method.clone();
    let target_url = data.url.to_owned();
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

    if !ssl_cert.is_empty() {
        let mut buf = Vec::new();
        fs::File::open(ssl_cert)
            .unwrap_or_else(|_| panic!("ERROR: Error reading {:?}", ssl_cert))
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
        HttpMethods::Get => {
            result = client.get(&*target_url).json(payload).send().await;
        }
        HttpMethods::Post => {
            result = client.post(&*target_url).json(payload).send().await;
        }
        HttpMethods::Put => {
            result = client.put(&*target_url).json(payload).send().await;
        }
        HttpMethods::Delete => {
            result = client.delete(&*target_url).json(payload).send().await;
        }
        HttpMethods::Patch => {
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
    // Generate command line args
    let matches = get_cmd_args().await;

    // Extract user supplied values
    let (output_format, config_filename, total_requests) = extract_values_from_args(matches).await;

    let shared_data: Data; // Data shared between tasks

    // Create RequestData from data file
    let request_data = get_request_data_from_file(config_filename).await;
    shared_data = Arc::new(request_data);

    // Variables shared between tasks
    let counter = Arc::new(Mutex::new(HashMap::new())); // Map of Atomic counters to keep HTTP status code count
    let (sender, receiver) = mpsc::channel(50);

    // Create a logger instance
    let logger = get_logger(LOG_PATH.to_owned()).await;
    let shared_logger = Arc::new(logger);

    // Start counter function
    let counter_clone = counter.clone();
    let counting_machine_handle =
        tokio::spawn(async move { counting_machine(counter_clone, receiver).await });

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
