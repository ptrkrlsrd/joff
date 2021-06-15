#![feature(decl_macro)]

use crate::http::StorableResponse;
use std::fs;
use clap::Clap;
use rocket::config::{Config, Environment};
use rocket::{Route, http::Method};

mod store;
mod url;
mod http;

#[derive(Clap)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(short, long, about = "Path to the KV store", default_value = "./data")]
    data_path: String,

    #[clap(short, long, about = "Name of the KV bucket", default_value = "json_data")]
    bucket_name: String,
    
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Add(Add),
    Serve(Serve),
    List(List),
}

#[derive(Clap)]
enum AddSubCommand {
    FromURL(AddFromURL),
    FromFile(AddFromFile),
}

#[derive(Clap)]
struct Add {
    #[clap(subcommand)]
    subcmd: AddSubCommand,
    #[clap(about = "Endpoint to map the stored data to. E.g /api/ditto")]
    local_endpoint: String,
}

#[derive(Clap)]
struct AddFromURL {
    #[clap(about = "URL to fetch data from. E.g https://pokeapi.co/api/v2/pokemon/ditto")]
    url: String,
}

#[derive(Clap)]
struct AddFromFile {
    #[clap(about = "File path")]
    path: String,
}

#[derive(Clap)]
struct Serve {
    #[clap(short, long, about = "Port to serve the Mock API on", default_value = "3000")]
    port: u16,

    #[clap(short, long, about = "Address for the Mock API", default_value = "127.0.0.1")]
    addr: String,

    #[clap(short, long, default_value = "30")]
    workers: u16,
}

#[derive(Clap)]
struct List { }

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let config_path: String = opts.data_path;
    let bucket_name: String = opts.bucket_name;

    let store = match store::new_store(config_path) {
        Ok(store) => store,
        Err(error) => panic!("Failed creating store: {:?}", error),
    };

    let bucket = match store::new_bucket(&store, &bucket_name) {
        Ok(bucket) => bucket,
        Err(error) => panic!("Failed creating bucket: {:?}", error),
    };

    match opts.subcmd {
        SubCommand::Add(add_args) => {
            let content: String;
            let encoded_url: String;
            let local_endpoint = add_args.local_endpoint;

            match add_args.subcmd {
                AddSubCommand::FromURL(url_args) => {
                    let url = url_args.url;
                    encoded_url = url::encode(&local_endpoint);

                    let reponse = http::get_json(&url).await.expect("Failed performing request");
                    content = reponse.to_string();
                },
                AddSubCommand::FromFile(path_args) => {
                    encoded_url = url::encode(&local_endpoint);
                    content = fs::read_to_string(path_args.path).expect("Could not read the file");
                }
            }

            store::set_value_for_key(&bucket, encoded_url, content)?;
        },
        SubCommand::List(_) => {
            store::list_items(&bucket);
        },
        SubCommand::Serve(args) => {
            let rocket_cfg = Config::build(Environment::Staging)
                .address(args.addr)
                .port(args.port)
                .workers(args.workers)
                .unwrap();

            let server = rocket::custom(rocket_cfg);
            let mut routes: Vec<Route> = Vec::new();

            for item in bucket.iter() {
                let key: String = match item?.key() {
                    Ok(key) => key,
                    Err(error) => panic!("Failed getting key: {:?}", error),
                };

                let bucket_data = match bucket.get(&key) {
                    Ok(data) => data,
                    Err(error) => panic!("Failed loading data: {:?}", error),
                };

                let response: StorableResponse = match serde_json::from_str(&bucket_data.unwrap()) {
                    Ok(json_data) => json_data,
                    Err(error) => panic!("Failed deserializing JSON: {:?}", error),
                };

                let decoded = match url::decode(&key) {
                    Ok(json_data) => json_data,
                    Err(error) => panic!("Failed decoding URL: {:?}", error),
                };

                let route = Route::new(Method::Get, &decoded, response);

                routes.push(route);
            }

            server.mount("/", routes).launch();
        }
    }

    Ok(())
}
