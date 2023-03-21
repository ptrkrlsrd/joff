use reqwest::header::HeaderMap;
use rocket::handler::{Handler, Outcome};
use rocket::Response;
use rocket::{Data, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use url::Url;

use percent_encoding::{percent_decode, utf8_percent_encode, AsciiSet, CONTROLS};

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;
type JsonValue = serde_json::Value;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorableResponse {
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl StorableResponse {
    fn from(body: String, headers: HashMap<String, String>) -> Self {
        return Self { body, headers }
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

pub async fn get_json(url: Url) -> Result<JsonValue, Error> {
    let response = reqwest::get(url).await?;
    let json = to_deserialized_response(response).await?;

    Ok(json)
}

fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    let headers = headers.iter()
    .map(|(key, value)| (key.to_string(), value.to_str().map_or(String::new(), str::to_string)))
    .collect::<HashMap<_, _>>();
    headers
}

async fn to_deserialized_response(resp: reqwest::Response) -> Result<JsonValue, Error> {
    let headers = resp.headers().clone();
    let json = read_json(resp).await?;

    let headers_map = headers_to_map(&headers);
    
    let storable_response = StorableResponse::from(json.to_string(), headers_map);
    let storable_response_json = serde_json::to_value(storable_response)?;

    Ok(storable_response_json)
}

async fn read_json(resp: reqwest::Response) -> Result<serde_json::Value, Error> {
    let json_data = resp.json::<serde_json::Value>().await?;
    let json = serde_json::from_str::<serde_json::Value>(&json_data.to_string())?;
    Ok(json)
}

pub fn encode_url(url: &String) -> String {
    let encoded_url_iter = utf8_percent_encode(&url, FRAGMENT);
    let encoded_url: String = encoded_url_iter.collect();

    return encoded_url;
}

pub fn decode_url(url: &str) -> Result<String, std::str::Utf8Error> {
    let decoded_iter = percent_decode(url.as_bytes());
    let decoded = decoded_iter.decode_utf8();

    return match decoded {
        Ok(url) => Ok(url.to_string()),
        Err(error) => Err(error),
    };
}
