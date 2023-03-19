use kv::*;
use std::result::Result;

use percent_encoding::{percent_decode, utf8_percent_encode, AsciiSet, CONTROLS};

type StorageError = kv::Error;

pub fn new_store(config_path: String) -> Result<kv::Store, StorageError> {
    let store_config = Config::new(config_path);

    return Store::new(store_config);
}

pub fn new_bucket<'a>(
    store: &Store,
    bucket_name: &str,
) -> Result<kv::Bucket<'a, String, String>, StorageError> {
    return store.bucket::<String, String>(Some(&bucket_name));
}

pub fn set_value_for_key(
    bucket: &Bucket<String, String>,
    key: String,
    value: String,
) -> Result<(), kv::Error> {
    return bucket.set(key, value);
}

pub fn list_items(bucket: &Bucket<String, String>) {
    for item in bucket.iter() {
        let key: String = item.unwrap().key().unwrap();
        let decoded = decode_url(&key).unwrap();

        println!("URL: {}", &decoded);
    }
}


const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

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