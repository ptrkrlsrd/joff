use std::collections::HashMap;
use std::io::Cursor;
use rocket::Response;
use url::Url;
use rocket::{Request, Data};
use rocket::handler::{Handler, Outcome};
use serde_json::json;
use serde::{Deserialize, Serialize};


type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorableResponse {
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl Handler for StorableResponse {
    fn handle<'r>(&self, req: &'r Request, _data: Data) -> Outcome<'r> {
        let mut response = Response::new();
        let disallowed_headers = vec!("transfer-encoding");

        for (key, value) in self.headers.iter() {
            if !disallowed_headers.contains(&key.as_str()) {
                response.set_raw_header(key.clone(), value.clone());
            }
        }

        response.set_sized_body(Cursor::new(self.body.clone()));
        return Outcome::from(req, response);
    }
}

pub async fn get_json(url: &String) -> Result<serde_json::Value> {
    let url = match Url::parse(&url) {
        Ok(url) => url,
        Err(error) => panic!("Failed parsing URL: {:?}", error),
    };

    let resp = match reqwest::get(url).await {
        Ok(resp) => resp,
        Err(error) => panic!("Failed getting response: {:?}", error),
    };

    let mut headers: HashMap<String, String> = HashMap::new();
    for (key, value) in resp.headers().iter() {
        let header_value = value.to_str()?;
        headers.insert(key.to_string(), header_value.to_string());
    }

    let data = match resp.json::<serde_json::Value>().await {
        Ok(json_data) => json_data,
        Err(error) => panic!("Failed deserializing JSON: {:?}", error),
    };

    let body = data.to_string();
    let storable_response = StorableResponse { body, headers };

    return Ok(json!(storable_response));
}
