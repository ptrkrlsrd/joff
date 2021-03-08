use kv::*;

pub fn new_store (config_path: String) -> Store{
    let store_config = Config::new(config_path);

    Store::new(store_config).unwrap()
}

pub fn new_bucket<'a>(store: &Store, bucket_name: &str) -> Bucket<'a, String, String> {
    store.bucket::<String, String>(Some(&bucket_name)).unwrap()
}

pub fn set_value_for_key(bucket: &Bucket<String, String>, key: String, value: String) {
    match bucket.set(key, value) {
        Ok(_i) => (),
        Err(err) => println!("{}", err),
    }
}