#[derive(Debug)]
pub enum Command {
    Increment(u16),
    Exit,
}

#[derive(Debug, Clone)]
pub enum HTTPMethods {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl HTTPMethods {
    pub fn fromstr(method: String) -> Option<HTTPMethods> {
        match method.as_str() {
            "GET" => Some(HTTPMethods::GET),
            "POST" => Some(HTTPMethods::POST),
            "PUT" => Some(HTTPMethods::PUT),
            "DELETE" => Some(HTTPMethods::DELETE),
            "PATCH" => Some(HTTPMethods::PATCH),
            _ => None,
        }
    }
}
