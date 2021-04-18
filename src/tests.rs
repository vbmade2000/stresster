#[cfg(test)]
mod tests {

    use serde_json::Value;
    use std::{
        env, fs,
        path::PathBuf,
        process::{Command, Stdio},
    };

    const STRESSTER_PATH: &str = "STRESSTER_PATH";
    const DATA_FILE_PATH: &str = "DATA_FILE_PATH";
    const LOG_FILE_PATH: &str = "LOG_FILE_PATH";

    /// Tests if stresster generates log file after execution
    #[test]
    fn log_file_exists() {
        // Get paths from env vars
        let stresster_path = get_path_from_env_var(STRESSTER_PATH.to_string());
        let data_file_path = get_path_from_env_var(DATA_FILE_PATH.to_string());
        let log_file_path = get_path_from_env_var(LOG_FILE_PATH.to_string());

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

        // Grab the output
        // let val: Value =
        //   serde_json::from_str(&String::from_utf8_lossy(&stresster_output.stdout)).unwrap();

        // Log file must be created
        assert_eq!(log_file_path.exists(), true);

        // Log file must not be empty
        assert_eq!(fs::metadata(log_file_path).unwrap().len() > 0, true)
    }

    /// Convert String in JSON format to Value
    fn _get_value_from_json(json: String) -> Value {
        serde_json::from_str(&json).unwrap()
    }

    /// Get PathBuf from path stored in ev var
    fn get_path_from_env_var(var_name: String) -> PathBuf {
        let val = env::var(var_name.to_string())
            .expect(format!("ERROR: {} env var is not present", var_name.to_string()).as_str());
        PathBuf::from(val)
            .canonicalize()
            .expect(format!("ERROR: {} path doesn't exist", var_name).as_str())
    }
}
