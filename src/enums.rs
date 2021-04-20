#[derive(Debug)]
pub enum Command {
    Increment(u16),
    Exit,
}

#[derive(Debug, Clone)]
pub enum HttpMethods {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethods {
    pub fn fromstr(method: String) -> Option<HttpMethods> {
        match method.as_str() {
            "get" => Some(HttpMethods::Get),
            "post" => Some(HttpMethods::Post),
            "put" => Some(HttpMethods::Put),
            "delete" => Some(HttpMethods::Delete),
            "patch" => Some(HttpMethods::Patch),
            _ => None,
        }
    }
}
