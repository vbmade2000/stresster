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
            "GET" => return Some(HTTPMethods::GET),
            "POST" => return Some(HTTPMethods::POST),
            "PUT" => return Some(HTTPMethods::PUT),
            "DELETE" => return Some(HTTPMethods::DELETE),
            "PATCH" => return Some(HTTPMethods::PATCH),
            _ => return None,
        };
    }
}
