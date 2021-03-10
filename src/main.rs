#![feature(decl_macro)]

use clap::Clap;
use rocket::config::{Config, Environment};
use rocket::{Route, http::Method};

mod store;
mod url;
mod http;

#[derive(Clap)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(short, long, default_value = "./data")]
    data_path: String,

    #[clap(short, long, default_value = "json_data")]
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
struct Add {
    source_url: String,
    alias_url: String,
}

#[derive(Clap)]
struct Serve {
    #[clap(short, long, default_value = "3000")]
    port: u16,

    #[clap(short, long, default_value = "127.0.0.1")]
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

    let store = store::new_store(config_path);
    let bucket = store::new_bucket(&store, &bucket_name);


    match opts.subcmd {
        SubCommand::Add(args) => {
            let url = args.source_url;
            let alias_url = args.alias_url;
            let encoded_url = url::encode(&alias_url);
            let reponse = http::get_json(&url).await?;

            store::set_value_for_key(&bucket, encoded_url, reponse.to_string());
        },
        SubCommand::List(_) => {
            for item in bucket.iter() {
                let key: String = item?.key()?;
                let decoded = url::decode(&key);

                println!("URL: {}", &decoded);
            }
        },
        SubCommand::Serve(args) => {
            let config = Config::build(Environment::Staging)
                .address(args.addr)
                .port(args.port)
                .workers(args.workers)
                .unwrap();

            let server = rocket::custom(config);
            let mut routes: Vec<Route> = Vec::new();

            for item in bucket.iter() {
                let key: String = item?.key()?;
                let data: String = bucket.get(&key)?.unwrap();

                let decoded = url::decode(&key);
                let route = Route::new(Method::Get, &decoded, http::JSONHandler{ data });

                routes.push(route);
            }

            server
                .mount("/", routes)
                .launch();
        }
    }

    Ok(())
}
