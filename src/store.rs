use kv::*;
use std::result::Result;

pub fn new_store (config_path: String) -> Result<kv::Store, kv::Error> {
    let store_config = Config::new(config_path);

    return Store::new(store_config);
}

pub fn new_bucket<'a>(store: &Store, bucket_name: &str) -> Result<kv::Bucket<'a, String, String>, kv::Error> {
    return store.bucket::<String, String>(Some(&bucket_name));
}

pub fn set_value_for_key(bucket: &Bucket<String, String>, key: String, value: String) -> Result<(), kv::Error> {
    return bucket.set(key, value);
}