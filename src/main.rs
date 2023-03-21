use crate::response::StorableResponse;
use std::fs;
use clap::Parser;
use reqwest::Url;
use rocket::config::{Config, Environment};
use rocket::{Route, http::Method};
use kv::Bucket;

mod storage;
mod response;

#[derive(Parser)]
#[command(version = "1.0")]
struct Opts {
    #[arg(short, long, default_value = "./data")]
    data_path: String,

    #[arg(short, long, default_value = "json_data")]
    bucket_name: String,
    
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Add(Add),
    Serve(Serve),
    List(List),
}

#[derive(Parser)]
enum AddSubCommand {
    FromURL(AddFromURL),
    FromFile(AddFromFile),
}

#[derive(Parser)]
struct Add {
    #[command(subcommand)]
    subcmd: AddSubCommand,
    #[arg()]
    local_endpoint: String,
}

#[derive(Parser)]
struct AddFromURL {
    #[arg(short, long)]
    url: String,
}

#[derive(Parser)]
struct AddFromFile {
    #[arg()]
    path: String,
}

#[derive(Parser)]
struct Serve {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(short, long, default_value = "127.0.0.1")]
    addr: String,

    #[arg(short, long, default_value = "30")]
    workers: u16,
}

#[derive(Parser)]
struct List { }

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let config_path: String = opts.data_path;
    let bucket_name: String = opts.bucket_name;

    let store = match storage::new_store(config_path) {
        Ok(store) => store,
        Err(error) => panic!("Failed creating store: {:?}", error),
    };

    let bucket = match storage::new_bucket(&store, &bucket_name) {
        Ok(bucket) => bucket,
        Err(error) => panic!("Failed creating bucket: {:?}", error),
    };

    match opts.subcmd {
        SubCommand::Add(add_args) => {
            match add_args.subcmd {
                AddSubCommand::FromURL(url_args) => {
                    add_from_url(bucket, add_args.local_endpoint, url_args.url).await;
                },
                AddSubCommand::FromFile(path_args) => {
                    add_from_file(bucket, add_args.local_endpoint, path_args.path).await;
                }
            }
        },
        SubCommand::List(_) => {
            storage::list_items(&bucket);
        },
        SubCommand::Serve(args) => {
            serve(bucket, args);
        }
    }

    Ok(())
}

async fn add_from_file(bucket: Bucket<'_, String, String>,local_endpoint: String, source_path: String) {
    let encoded_url = storage::encode_url(&local_endpoint);
    let content = match fs::read_to_string(source_path) {
        Ok(content) => content,
        Err(error) => panic!("Failed reading file: {:?}", error),
    };

    let _ = storage::set_value_for_key(&bucket, encoded_url, content);
}

async fn add_from_url(bucket: Bucket<'_, String, String>, local_endpoint: String, source_url: String) {
    let url = match Url::parse(&source_url) {
        Ok(url) => url,
        Err(error) => panic!("Failed parsing URL: {:?}", error),
    };

    let response = response::get_json(url).await;
    let response = match response {
        Ok(response) => response,
        Err(error) => panic!("Failed getting JSON from error: {:?}", error),
    };

    let encoded_url = storage::encode_url(&local_endpoint);
    let _ = storage::set_value_for_key(&bucket, encoded_url, response.to_string());
}

fn serve(bucket: Bucket<String, String>, args: Serve) {
    let rocket_cfg = Config::build(Environment::Staging)
        .address(args.addr)
        .port(args.port)
        .workers(args.workers)
        .unwrap();

    let server = rocket::custom(rocket_cfg);
    let routes: Vec<Route> = bucket.iter().filter_map(|item| {
        let key: String = item.unwrap().key().unwrap();
    
        let bucket_data = match bucket.get(&key) {
            Ok(data) => data,
            Err(error) => {
                println!("Failed getting data for key: {:?}, error: {:?}", key, error);
                return None;
            }
        };
    
        let json_response: StorableResponse = match serde_json::from_str(&bucket_data.unwrap()) {
            Ok(json_data) => json_data,
            Err(error) => {
                println!("Failed deserializing JSON for key: {:?}, error: {:?}", key, error);
                return None;
            }
        };
    
        let decoded_url = match storage::decode_url(&key) {
            Ok(url) => url,
            Err(error) => {
                println!("Failed decoding key to URL: {:?}, error: {:?}", key, error);
                return None;
            }
        };
    
        let route = Route::new(Method::Get, &decoded_url, json_response);
    
        Some(route)
    }).collect();

    server.mount("/", routes).launch();

}
