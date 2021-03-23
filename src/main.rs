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

    let store = store::new_store(config_path)?;
    let bucket = store::new_bucket(&store, &bucket_name)?;

    match opts.subcmd {
        SubCommand::Add(add_args) => {
            let content: String;
            let encoded_url: String;
            let local_endpoint = add_args.local_endpoint;

            match add_args.subcmd {
                AddSubCommand::FromURL(url_args) => {
                    let url = url_args.url;
                    encoded_url = url::encode(&local_endpoint);

                    let reponse = http::get_json(&url).await?;
                    content = reponse.to_string();
                },
                AddSubCommand::FromFile(path_args) => {
                    encoded_url = url::encode(&local_endpoint);
                    content = fs::read_to_string(path_args.path)
                        .expect("Could not read the file");
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
                let key: String = item?.key()?;
                let bucket_data = bucket.get(&key)?;

                let v: StorableResponse = serde_json::from_str(&bucket_data.unwrap())?;
                let decoded = url::decode(&key)?;
                let route = Route::new(Method::Get, &decoded, http::StorableResponse{ body: v.body, headers: v.headers });

                routes.push(route);
            }

            server.mount("/", routes).launch();
        }
    }

    Ok(())
}
