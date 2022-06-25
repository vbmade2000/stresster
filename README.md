### What is stresster?
***stresster*** is a command line utility to stress test REST API endpoints. It is developed using a Rust programming language and uses tokio async runtime to concurrently send requests. This makes it fast.

### Features
1. Control number of requests sent
2. Send HTTP Headers
3. SSPL/TLS Certificate
4. All the major HTTP methods (GET, POST, PUT, PATCH, DELETE) are supported
5. Send JSON payload

### Warning
It is being developed as a hobby project to learn Rust so use it at your own risk.

### Steps to compile from source
`$ git clone https://github.com/vbmade2000/stresster.git`  
`$ cd stresster`  
`$ cargo build --release`  
`$ cargo run`  
You should see help text on screen.
```
error: The following required arguments were not provided:
    --config <config>

USAGE:
    stresster --config <config> --format <format> --requests <total_requests>

For more information try --help
```

### How to use it?
Specify required parameters related to HTTP requests (headers, payload etc) in a file in a JSON format.

##### Supported fields in data.json
1. ***url***: A REST API URL to test (mandatory)
2. ***ssl_cert***: SSL/TLS certificate (.cert file) to connect to secure URL (optional)
3. ***method***: HTTP Method to be used for request
4. ***payload***: A JSON object containing a payload to be sent with request (optional)
5. ***headers***: A JSON object containing HTTP headers in the form of ***key: value*** pairs. Case doesn't matter here. (optional)

##### Command line arguments
1. ***--config***: A file containing a requested related data in JSON format. (mandatory)
2. ***--format***: AN output format. Default is ***table*** but you can see output in ***json*** format too. (optional).
3. ***--requests***: Total number of requests to send. (mandatory)

#### Example command
`cargo run -- --requests 5 -c payload.json`  
Here target server is not up so status code is 0.
```
+-------------+-------+
| Status Code | Count |
+-------------+-------+
| 0           | 5     |
+-------------+-------+
```
### Sample payload
```
{
  "url": "http://localhost:15000/gettest",
  "ssl_cert": "./test_server/cert.pem",
  "method": "get",
  "payload": {
    "name": "Malhar Vora"
  },
  "headers": {
        "User-Agent": "stresster",
        "Content-Type": "application/json"
  }
}
```


### Run integration tests

Integration tests require test server (developed using Python) to run. YOu can run in following way if not already running.
```
$ python3 -m venv venv
$ source venv/bin/activate
$ pip3 install flask
$ cd test_server
$ python server.py
```

If values for following variables are different from shown below then you can export env vars with same name.
```
STRESSTER_PATH=./target/debug/stresster
DATA_FILE_PATH=./sample_payload.json
LOG_FILE_PATH=./stresster.log
```

`cargo build`
`cargo test -- --nocapture`
`pkill -9 -f 'python server.py'`


### Report bugs etc 
[Malhar Vora](mailto://vbmade2000@gmail.com)
