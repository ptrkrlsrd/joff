use kv::{Config, Bucket, Store};
use reqwest::Url;
use std::{result::Result, fs};


use crate::response::{self, encode_url};

type StorageError = kv::Error;

pub fn new_store(config_path: String) -> Result<kv::Store, StorageError> {
    let store_config = Config::new(config_path);

    return Store::new(store_config);
}

pub fn new_bucket<'a>(store: &Store, bucket_name: &str) -> Result<kv::Bucket<'a, String, String>, StorageError> {
    return store.bucket::<String, String>(Some(&bucket_name));
}

pub fn set_value_for_key(bucket: &Bucket<String, String>, key: String, value: String) -> Result<(), kv::Error> {
    return bucket.set(key, value);
}

pub fn list_items(bucket: &Bucket<String, String>) {
    for item in bucket.iter() {
        let key: String = item.unwrap().key().unwrap();
        let decoded = response::decode_url(&key).unwrap();

        println!("URL: {}", &decoded);
    }
}

pub fn add_from_file(bucket: Bucket<'_, String, String>, alias_enpoint: String, file_path: String) {
    let encoded_url = encode_url(&alias_enpoint);
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(error) => panic!("Failed reading file: {:?}", error),
    };

    let _ = set_value_for_key(&bucket, encoded_url, content);
}

pub async fn add_from_url(bucket: Bucket<'_, String, String>, alias_endpoint: String, source_url: String) {
    let url = match Url::parse(&source_url) {
        Ok(url) => url,
        Err(error) => panic!("Failed parsing URL: {:?}", error),
    };

    let response = response::get_json(url).await;
    let response = match response {
        Ok(response) => response,
        Err(error) => panic!("Failed getting JSON from error: {:?}", error),
    };

    let encoded_url = encode_url(&alias_endpoint);
    let _ = set_value_for_key(&bucket, encoded_url, response.to_string());
}

