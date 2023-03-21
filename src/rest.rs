use reqwest::header::HeaderMap;
use rocket::handler::{Handler, Outcome};
use rocket::Response;
use anyhow::Result;
use rocket::{Data, Request};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use url::Url;

use percent_encoding::{percent_decode, utf8_percent_encode, AsciiSet, CONTROLS};

type Error = Box<dyn std::error::Error>;
type JsonValue = serde_json::Value;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorableResponse {
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl StorableResponse {
    pub fn from(body: String, headers: HashMap<String, String>) -> Self {
        return Self { body, headers }
    }


    pub fn from_json(data: String) -> Result<StorableResponse, serde_json::Error> {
        let json_response: StorableResponse = match serde_json::from_str(&data) {
            Ok(json_data) => json_data,
            Err(error) => {
                return Err(error);
            }
        };

        Ok(json_response)
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

pub struct RestClient;

impl RestClient {
    pub async fn get_json(url: Url) -> Result<JsonValue, Error> {
        let response = reqwest::get(url).await?;
        let headers = response.headers().clone();
        let headers = Self::headers_to_map(&headers);

        let json = Self::to_serialized_response(response, headers).await?;

        Ok(json)
    }

    async fn to_serialized_response(resp: reqwest::Response, headers: HashMap<String, String>) -> Result<JsonValue, Error> {
        let json_response = Self::read_json_from_response(resp).await?;
        let storable_response = StorableResponse::from(json_response.to_string(), headers);

        let storable_response_json = serde_json::to_value(storable_response)?;
        Ok(storable_response_json)
    }

    fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
        headers.iter()
            .filter_map(|(key, value)| {
                value.to_str().ok().map(|value_str| (key.to_string(), value_str.to_string()))
            })
        .collect::<HashMap<_, _>>()
    }

    async fn read_json_from_response(resp: reqwest::Response) -> Result<JsonValue, Error> {
        let json_data = resp.json::<serde_json::Value>().await?;
        Ok(json_data)
    }
}

pub fn encode_url(url: &String) -> String {
    let encoded_url = utf8_percent_encode(&url, FRAGMENT);
    let encoded_url: String = encoded_url.collect();

    return encoded_url;
}

pub fn decode_url(url: &str) -> Result<String, std::str::Utf8Error> {
    let decoded_url = percent_decode(url.as_bytes());
    let decoded_url = decoded_url.decode_utf8();

    return match decoded_url {
        Ok(url) => Ok(url.to_string()),
        Err(error) => Err(error),
    };
}
