use crate::http::StorableResponse;
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
                let key: String = item?.key().expect("Failed getting key");

                let bucket_data = bucket.get(&key).expect("Failed loading data");

                let response: StorableResponse = serde_json::from_str(&bucket_data.unwrap())
                    .expect("Failed deserializing JSON");

                let decoded = url::decode(&key).expect("Failed decoding URL");
                let route = Route::new(Method::Get, &decoded, response);

                routes.push(route);
            }

            server.mount("/", routes).launch();
        }
    }

    Ok(())
}
