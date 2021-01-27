use std::sync::{Arc, Mutex};
use std::fs;
use std::process::exit;

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

    let filename = "payload.json".to_string();
    let result = fs::read_to_string(filename.to_string());
    let content:String;
    match result {
        Ok(r) => content = r,
        Err(e) => {
            println!("ERROR: {}, {}", filename.to_string(), e);
            exit(1)
        }
    }; 
    

    let url = "http://0.0.0.0:15000/posttest".to_string();
    let shared_url = Arc::new(Mutex::new(url));

    loop {
        let shared_url = shared_url.clone();  
        tokio::spawn(async move { send(shared_url).await });
    }

    // https://api.github.com/users/vbmade2000
}
