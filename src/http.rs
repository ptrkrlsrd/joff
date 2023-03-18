use reqwest::header::HeaderMap;
use rocket::handler::{Handler, Outcome};
use rocket::Response;
use rocket::{Data, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use url::Url;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorableResponse {
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl StorableResponse {
    fn from(body: String, headers: HashMap<String, String>) -> StorableResponse {
        return StorableResponse { body, headers }
    }
}

impl Handler for StorableResponse {
    fn handle<'r>(&self, req: &'r Request, _data: Data) -> Outcome<'r> {
        let mut response = Response::new();
        let disallowed_headers = vec!["transfer-encoding"];

        for (key, value) in self.headers.iter() {
            if !disallowed_headers.contains(&key.as_str()) {
                response.set_raw_header(key.clone(), value.clone());
            }
        }

        response.set_sized_body(Cursor::new(self.body.clone()));
        return Outcome::from(req, response);
    }
}

pub async fn get_json(url: Url) -> Result<serde_json::Value, Error> {
    let resp = match reqwest::get(url).await {
        Ok(resp) => resp,
        Err(error) => return Err(error.into()),
    };

    let headers = headers_to_map(resp.headers().clone());

    let json = resp.json::<serde_json::Value>().await?;

    let storable_response = StorableResponse::from(json.to_string(), headers);

    Ok(serde_json::to_value(storable_response)?)
}

pub fn headers_to_map(headers: HeaderMap) -> HashMap<String, String> {
    let headers = headers.iter()
    .map(|(key, value)| (key.to_string(), value.to_str().map_or(String::new(), str::to_string)))
    .collect::<HashMap<_, _>>();
    headers
}
