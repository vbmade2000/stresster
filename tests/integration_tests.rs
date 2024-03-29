use serde_json::{from_str, json, Value};
use std::str;
use std::{
    env,
    env::temp_dir,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};
use uuid::Uuid;

const STRESSTER_PATH: &str = "STRESSTER_PATH";
const DATA_FILE_PATH: &str = "DATA_FILE_PATH";
const LOG_FILE_PATH: &str = "LOG_FILE_PATH";

/// Tests if stresster generates log file after execution
#[test]
fn log_file_exists() {
    // Get paths from env vars
    let stresster_path = get_path_from_env_var(
        STRESSTER_PATH.to_string(),
        "./target/debug/stresster".to_string(),
        true,
    );
    let data_file_path = get_path_from_env_var(
        DATA_FILE_PATH.to_string(),
        "./sample_payload.json".to_string(),
        true,
    );

    let log_file_path = get_path_from_env_var(
        LOG_FILE_PATH.to_string(),
        "./stresster.log".to_string(),
        false,
    );

    // Delete existing strestter log file (if exists) to generate fresh one
    let _ = fs::remove_file(log_file_path.clone());

    // Execute stresster
    let _ = Command::new(stresster_path)
        .arg("--config")
        .arg(data_file_path.to_str().unwrap())
        .arg("--requests")
        .arg("2")
        .arg("--format")
        .arg("json")
        .stdout(Stdio::piped())
        .output()
        .expect("ERROR: Error in executing stresster binary");

    // Log file must be created
    assert_eq!(log_file_path.exists(), true);

    // Log file must not be empty
    assert_eq!(fs::metadata(log_file_path).unwrap().len() > 0, true)
}

/// Tests if headers from payload file are sent successfully
#[test]
fn test_headers() {
    // Get paths from env vars
    let stresster_path = get_path_from_env_var(
        STRESSTER_PATH.to_string(),
        "./target/debug/stresster".to_string(),
        true,
    );

    /* We need to create a new data file from original file. It will contain an
     * extra header we want to supply. This way we keep original file intact.
     * We create new file with a UUID as a name to keep it unique and also in new
     * temporary directry to avoid conflicts.
     */
    let dir = temp_dir();
    let mut temp_file_name = PathBuf::new();
    temp_file_name.push(dir);
    temp_file_name.push(format!("{}.json", Uuid::new_v4().to_string()));

    // Read JSON from file
    // let mut file = fs::File::open(temp_file_name).unwrap();
    let data = json!({
        "url": "http://localhost:15000/gettest",
        "ssl_cert": "./test_server/cert.pem",
        "method": "get",
        "payload": {
            "name": "Malhar Vora"
        },
        "headers": {
            "User-Agent": "stresster",
            "Content-Type": "application/json",
            "code": "204"
        }
    });

    //file.write_all(data.as_str().unwrap());
    let _ = serde_json::to_writer(
        &fs::File::create(temp_file_name.to_string_lossy().to_string()).expect(
            format!(
                "Unable to create temporary data file {}",
                temp_file_name.to_string_lossy()
            )
            .as_str(),
        ),
        &data,
    );

    // Execute stresster
    let output = Command::new(stresster_path)
        .arg("--config")
        .arg(temp_file_name.to_str().unwrap())
        .arg("--requests")
        .arg("1")
        .arg("--format")
        .arg("json")
        .stdout(Stdio::piped())
        .output()
        .expect("ERROR: Error in executing stresster binary");

    // Extract total failed request from output
    let output: Value = from_str(str::from_utf8(&output.stdout).unwrap())
        .expect("Unable to convert stresster output to JSON");

    // Here We check if that one request we have received has status code 204 that we sent with request for testing purpose.
    let recived_status_code = output.get("204").unwrap();
    assert_eq!(recived_status_code, 1);
}

/// Convert String in JSON format to Value
fn _get_value_from_json(json: String) -> Value {
    from_str(&json).unwrap()
}

/// Get PathBuf from path stored in ev var
fn get_path_from_env_var(var_name: String, default_value: String, must_present: bool) -> PathBuf {
    let val = env::var(var_name.to_string()).unwrap_or(default_value);
    if must_present {
        PathBuf::from(val).canonicalize().unwrap()
    } else {
        PathBuf::from(val)
    }
}
