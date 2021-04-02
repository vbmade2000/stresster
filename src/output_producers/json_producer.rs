use crate::types::COUNTERMAP;

/// Shows output in JSON format. It is helpful when stresser is useful in automation
pub async fn produce_json_output(counter_map: COUNTERMAP) {
    let map = counter_map.lock().await;

    // Create nice JSON using serde
    let serialized_json = serde_json::to_string_pretty(&*map);
    println!("{}", serialized_json.unwrap());
}
