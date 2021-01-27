use std::sync::{Arc, Mutex};

async fn send(url: URL) {
    let target_url = url.lock().unwrap().clone();
    let result = reqwest::get(&target_url).await;
    match result {
        Ok(r) => println!("Found something"),
        Err(_) => println!("Found error"),
    }
}

type URL = Arc<Mutex<String>>; 

#[tokio::main]
async fn main() {

    let url = "0.0.0.0:15000".to_string();
    let shared_url = Arc::new(Mutex::new(url));

    loop {
        let shared_url = shared_url.clone();  
        tokio::spawn(async move { send(shared_url).await });
    }

    // https://api.github.com/users/vbmade2000
}
