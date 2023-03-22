use kv::{Config, Bucket, Store, Item};
use reqwest::Url;
use rocket::{Route, http::Method};
use std::{result::Result, fs};
use crate::rest::{StorableResponse, self, encode_url, decode_url};

type StorageError = kv::Error;
type Error = Box<dyn std::error::Error>;

pub struct RouteManager<'a> {
    pub bucket: Bucket<'a, String, String>
}

impl RouteManager<'_> {
    pub fn new(config_path: String, bucket_name: String) -> Result<Self, StorageError> {
        let store = Self::new_storage(config_path)?;
        let bucket = Self::new_bucket(&store, &bucket_name)?;

        Ok(Self { bucket })
    }

    pub fn new_route_from_file(self, alias_enpoint: String, file_path: String) {
        let encoded_url = encode_url(&alias_enpoint);
        let content = fs::read_to_string(file_path).expect("Failed reading file");

        Self::set_value_for_key(&self.bucket, encoded_url, content).expect("Failed setting value for key");
    }

    pub async fn new_route_from_url(self, alias_endpoint: String, source_url: String) {
        let url = Url::parse(&source_url).expect("Failed parsing URL");

        let response = rest::RestClient::get_json(url).await.expect("Failed getting JSON from error");

        let encoded_url = encode_url(&alias_endpoint);
        Self::set_value_for_key(&self.bucket, encoded_url, response.to_string()).expect("Failed setting value for key");
    }

    pub fn new_storage(config_path: String) -> Result<kv::Store, StorageError> {
        let store_config = Config::new(config_path);
        Store::new(store_config)
    }

    pub fn clean_storage(self) -> Result<usize, StorageError> {
        self.bucket.flush()
    }

    pub fn new_bucket<'a>(store: &Store, bucket_name: &str) -> Result<kv::Bucket<'a, String, String>, StorageError> {
        store.bucket::<String, String>(Some(&bucket_name))
    }

    pub fn set_value_for_key(bucket: &Bucket<String, String>, key: String, value: String) -> Result<(), StorageError> {
        bucket.set(key, value)
    }

    pub fn list_items(self) -> Result<(), Error> {
        for item in self.bucket.iter() {
            let key: String = item?.key()?;
            let decoded = rest::decode_url(&key)?;

            println!("URL: {}", &decoded);
        }

        Ok(())
    }

    pub fn get_routes_from_bucket(self) -> Vec<Route> {
        self.bucket.iter().filter_map(|item| {
            let item = item.unwrap();
            let key = Self::get_key(&item)?;
            let route = Self::new_route(&self.bucket, key).unwrap();

            Some(route)
        }).collect()
    }

    fn get_key(item: &Item<String, String>) -> Option<String> {
        match item.key() {
            Ok(key) => Some(key),
            Err(error) => {
                println!("Failed getting key: {:?}", error);
                None
            }
        }
    }

    fn new_route(bucket: &Bucket<String, String>, url: String) -> Result<Route, Error> {
        let bucket_data = bucket.get(&url)?.unwrap();

        let json_response = StorableResponse::from_json(bucket_data)?;
        let decoded_url = decode_url(&url)?;

        let route = Route::new(Method::Get, &decoded_url, json_response);

        Ok(route)
    }
}
