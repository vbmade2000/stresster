extern crate clap;
use clap::{App, Arg, SubCommand};
use std::fs;
use std::process::exit;
use std::sync::{Arc, Mutex};

async fn send(url: URL) {
    let target_url = url.lock().unwrap().clone();
    let client = reqwest::Client::new();
    let js = r#"{
        "name": "Malhar Vora"
    }"#;
    let result = client.post(&target_url).json(js).send().await;
    match result {
        Ok(r) => println!("Found something"),
        Err(e) => println!("{:?}", e),
    }
}

type URL = Arc<Mutex<String>>;

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

    let result = fs::read_to_string(payload_filename.to_string());
    let content: String;
    match result {
        Ok(r) => content = r,
        Err(e) => {
            println!("ERROR: {}, {}", payload_filename.to_string(), e);
            exit(1)
        }
    };

    println!("{}", content);
    return;

    let url = "http://0.0.0.0:15000/posttest".to_string();
    let shared_url = Arc::new(Mutex::new(url));

    loop {
        let shared_url = shared_url.clone();
        tokio::spawn(async move { send(shared_url).await });
    }

    // https://api.github.com/users/vbmade2000
}
