use crate::http::StorableResponse;
use std::f32::consts::E;
use std::fs;
use clap::{Parser};
use rocket::config::{Config, Environment};
use rocket::{Route, http::Method};

mod store;
mod url;
mod http;

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

                    let reponse = http::get_json(&url).await;
                    let resposnse = match reponse {
                        Ok(response) => response,
                        Err(error) => panic!("Failed getting JSON from URL: {:?}, error: {:?}", url, error),
                    };
                    content = reponse.to_string();
                },
                AddSubCommand::FromFile(path_args) => {
                    encoded_url = url::encode(&local_endpoint);
                    content = match fs::read_to_string(path_args.path) {
                        Ok(content) => content,
                        Err(error) => panic!("Failed reading file: {:?}", error),
                    }
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
                    Some(key) => key.to_string(),
                    None => {
                        println!("No key found for item: {:?}", item);
                        continue;
                    }
                };

                let bucket_data = match bucket.get(&key) {
                    Ok(data) => data,
                    Err(error) => {
                        println!("Failed getting data for key: {:?}, error: {:?}", key, error);
                        continue;
                    }
                };

                let bucket_data = match bucket_data {
                    Some(data) => data,
                    None => {
                        println!("No data found for key: {:?}", key);
                        continue;
                    }
                };

                let response: StorableResponse = match serde_json::from_str(&bucket_data) {
                    Ok(response) => response,
                    Err(error) => {
                        println!("Failed deserializing JSON for key: {:?}, error: {:?}", key, error);
                        continue;
                    }
                };

                let decoded = match url::decode(&key) {
                    Ok(decoded) => decoded,
                    Err(error) => {
                        println!("Failed decoding key to URL: {:?}, error: {:?}", key, error);
                        continue;
                    }
                };

                let route = Route::new(Method::Get, &decoded, response);

                routes.push(route);
            }

            server.mount("/", routes).launch();
        }
    }

    Ok(())
}
