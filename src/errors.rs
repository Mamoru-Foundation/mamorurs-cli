use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
    //extensions: Extensions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extensions {
    code: String,
    //response: Response,
    //service_name: String,
    //exception: Exception,
    //stacktrace: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    status_code: u16,
    message: String,
    error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Exception {
    message: String,
    stacktrace: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub errors: Option<Vec<Error>>,
}
