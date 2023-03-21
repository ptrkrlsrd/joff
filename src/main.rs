use clap::Parser;
use kv::Bucket;
use kv::Item;
use crate::response::decode_url;
use crate::response::StorableResponse;
use storage::RouteManager;
use rocket::{config::{Config, Environment}, http::Method, Route};

mod storage;
mod response;

#[derive(Parser)] // requires `derive` feature
#[command(name = "joff")]
#[command(version = "1.0")]
#[command(about = "jepp", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "./data")]
    data_path: String,

    #[arg(short, long, default_value = "json_data")]
    bucket_name: String,
    
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    #[command(arg_required_else_help = true)]
    Add(Add),
    Serve(Serve),
    List(List),
}

#[derive(Parser)]
#[command(args_conflicts_with_subcommands = true)]
struct Add {
    #[command(subcommand)]
    subcmd: AddSubCommand,
}

#[derive(Parser)]
enum AddSubCommand {
    FromURL(AddFromURL),
    FromFile(AddFromFile),
}

#[derive(Parser)]
struct AddFromURL {
    #[arg(required = true)]
    url: String,

    #[arg(required = true)]
    alias_url: String,
}

#[derive(Parser)]
struct AddFromFile {
    #[arg(required = true)]
    file_path: String,

    #[arg(required = true)]
    alias_url: String,
}

#[derive(Parser)]
struct Serve {
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(short, long, default_value = "127.0.0.1")]
    addr: String,

    #[arg(short, long, default_value = "30")]
    workers: u16,

    #[arg(short, long, default_value = "/")]
    base_endpoint: String,
}

#[derive(Parser)]
struct List;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Cli = Cli::parse();
    let config_path: String = opts.data_path;
    let bucket_name: String = opts.bucket_name;

    let store = match RouteManager::new_store(config_path) {
        Ok(store) => store,
        Err(error) => panic!("Failed creating store: {:?}", error),
    };

    let bucket = match RouteManager::new_bucket(&store, &bucket_name) {
        Ok(bucket) => bucket,
        Err(error) => panic!("Failed creating bucket: {:?}", error),
    };

    match opts.subcmd {
        SubCommand::Add(add_args) => {
            match add_args.subcmd {
                AddSubCommand::FromURL(url_args) => {
                    RouteManager::new_route_from_url(bucket, url_args.alias_url, url_args.url).await;
                },
                AddSubCommand::FromFile(path_args) => {
                    RouteManager::new_route_from_file(bucket, path_args.alias_url, path_args.file_path);
                }
            }
        },
        SubCommand::List(_) => {
            let _ = RouteManager::list_items(&bucket);
        },
        SubCommand::Serve(args) => {
            let _ = serve(bucket, args);
        }
    }

    Ok(())
}

fn serve(bucket: Bucket<String, String>, args: Serve) {
    let rocket_cfg = Config::build(Environment::Staging)
        .address(args.addr)
        .port(args.port)
        .workers(args.workers)
        .unwrap();

    let server = rocket::custom(rocket_cfg);
    let routes = get_routes_from_bucket(bucket);

    server.mount(args.base_endpoint.as_str(), routes).launch();
}

fn get_routes_from_bucket(bucket: Bucket<String, String>) -> Vec<Route> {
    bucket.iter().filter_map(|item| {
        let item = item.unwrap();
        let key = get_key(&item)?;
        let route = new_route(&bucket, key);
        route
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

fn new_route(bucket: &Bucket<String, String>, key: String) -> Option<Route> {
    let json_response = route_from_bucket(&bucket, &key)?;
    let decoded_url = match decode_url(&key) {
        Ok(url) => url,
        Err(error) => {
            println!("Failed decoding key to URL: {:?}, error: {:?}", key, error);
            return None;
        }
    };
    let route = Route::new(Method::Get, &decoded_url, json_response);

    Some(route)
}

fn route_from_bucket(bucket: &Bucket<String, String>, key: &String) -> Option<StorableResponse> {
    let bucket_data = match bucket.get(key) {
        Ok(data) => data,
        Err(error) => {
            println!("Failed getting data for key: {:?}, error: {:?}", key, error);
            return None;
        }
    }?;

    let json_response: StorableResponse = match serde_json::from_str(&bucket_data) {
        Ok(json_data) => json_data,
        Err(error) => {
            println!("Failed deserializing JSON for key: {:?}, error: {:?}", key, error);
            return None;
        }
    };

    Some(json_response)
}
