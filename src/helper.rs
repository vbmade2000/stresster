use crate::output_producers::output_producer::OutputProducer;
use crate::output_producers::{json_producer, table_producer};
use crate::types::{HttpMethods, OutputFormat, RequestData};
use clap::{App, Arg, ArgMatches};
use reqwest::header::{HeaderName, HeaderValue};
use serde_json::Value;
use slog::{Drain, Logger};
use std::fs;
use std::process::exit;

/// Extracts and returns all the command line parameters
pub async fn extract_values_from_args(args: ArgMatches<'_>) -> (OutputFormat, String, i32) {
    // Extract user supplied values
    let output_format = OutputFormat::from(args.value_of("format").unwrap());
    let config_filename = args.value_of("config").unwrap().to_owned();
    let total_requests: i32 = args.value_of("requests").unwrap().parse().unwrap();
    (output_format, config_filename, total_requests)
}

/// Reads and parses Data file and returns RequestData struct with values fufilled
pub async fn get_request_data_from_file(config_filename: &str) -> RequestData {
    let content: Value; // Stores JSON data read from file

    // Default values in case actual values are not supplied
    let default_value: serde_json::Value = serde_json::from_str("{}").unwrap();

    // TODO: Make error handling compact
    let result = fs::read_to_string(&config_filename);
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

    // Extract payload or get default payload
    let actual_payload = content.get("payload").unwrap_or(&default_value);
    let mut request_data = RequestData {
        payload: actual_payload.clone(),
        ..Default::default()
    };

    // Extract HTTP headers or get default HTTP headers
    let headers = content
        .get("headers")
        .unwrap_or(&default_value)
        .as_object()
        .unwrap();

    // Insert extracted headers to shared object
    for (key, value) in headers {
        request_data.headers.insert(
            HeaderName::from_lowercase(key.to_lowercase().as_bytes()).unwrap(),
            HeaderValue::from_str(value.as_str().unwrap()).unwrap(),
        );
    }

    // Extract HTTP method
    let method = content
        .get("method")
        .expect("ERROR: Please specify method in payload file")
        .as_str()
        .unwrap();

    // Grab enum value from string HTTP method
    let http_method = HttpMethods::fromstr(method);
    if http_method.is_none() {
        println!("ERROR: Invalid HTTP method {:?}", method);
        exit(1);
    }
    request_data.method = http_method.unwrap();

    // Extract URL
    request_data.url = content
        .get("url")
        .expect("ERROR: Please specify URL in payload file")
        .as_str()
        .unwrap()
        .to_owned();

    // Extract cert_path if spplied
    request_data.cert_path = content
        .get("ssl_cert")
        .unwrap_or(&default_value)
        .as_str()
        .unwrap_or("")
        .to_owned();

    request_data
}

/// Creates a new logger and returns it
pub async fn get_logger(filename: &str) -> Logger {
    // Create a logger instance
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
        .expect("ERROR: Unable to create a log file");
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_async::Async::new(slog_term::FullFormat::new(decorator).build().fuse())
        .build()
        .fuse();
    slog::Logger::root(drain, o!())
}

/// Specifies all the command line arguments. Constructs a
/// ArgMatches instance and returns it.
pub async fn get_cmd_args<'a>() -> ArgMatches<'a> {
    App::new("stresster")
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
        .get_matches()
}

/// Returns an output producer based on `OutputFormat`
pub async fn get_output_producer(output_format: OutputFormat) -> Box<dyn OutputProducer> {
    match output_format {
        OutputFormat::Json => Box::new(json_producer::JSONProducer {}),
        OutputFormat::Table => Box::new(table_producer::TableProducer {}),
    }
}
