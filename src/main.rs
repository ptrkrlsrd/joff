#![feature(decl_macro)]

use reqwest::Body;
use core::str::Bytes;
use clap::Clap;
use kv::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use url::Url;
use percent_encoding::{utf8_percent_encode, percent_decode, AsciiSet, CONTROLS};
use std::str::Utf8Error;
use rocket::handler::Outcome;
use rocket::{Request, Route, Data};
use rocket::http::Method;

#[macro_use] extern crate rocket;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');


#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
struct Opts {
    #[clap(short, long, default_value = "default.conf")]
    config: String,
    #[clap(short, long, default_value = "./data")]
    data_path: String,
    #[clap(short, long, parse(from_occurrences))]
    _verbose: i32,
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
}

#[derive(Clap)]
struct List {
}

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

async fn perform_request(url: &String) -> Result<serde_json::Value> {
    let url = Url::parse(&url)?;

    let resp = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    return Ok(resp);
}

fn handler<'r>(request: &'r Request, _data: Data) -> Outcome<'r> {
    Outcome::from(request, "Hello, world!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let config_path: String = opts.data_path;

    let store_config = Config::new(config_path);
    let store = Store::new(store_config)?;
    let bucket_name = "acache";
    let bucket = store.bucket::<String, String>(Some(bucket_name))?;

    match opts.subcmd {
        SubCommand::Add(t) => {
            let url = t.source_url;
            let alias_url = t.alias_url;
            let reponse = perform_request(&url).await?;

            let encoded_url_iter = utf8_percent_encode(&alias_url, FRAGMENT);
            let _encoded_url: String = encoded_url_iter.collect();


            match bucket.set(_encoded_url, reponse.to_string()) {
                Ok(_i) => (),
                Err(err) => println!("{}", err),
            }
        },
        SubCommand::List(_t) => {
            for item in bucket.iter() {
                let item = item?;
                let key: String = item.key()?;
                let _data: String = bucket.get(&key)?.unwrap();

                let iter = percent_decode(key.as_bytes());
                let decoded = iter.decode_utf8()?;
                println!("key: {}, {}", key, &decoded);
            }
        },
        SubCommand::Serve(_t) => {
            let mut server = rocket::ignite();

            for item in bucket.iter() {
                let item = item?;
                let key: String = item.key()?;
                let data: String = bucket.get(&key)?.unwrap();

                let iter = percent_decode(key.as_bytes());
                let decoded = iter.decode_utf8()?;
                let route = Route::new(Method::Get, &decoded, handler);

                server = server.mount("/", vec!(route));
            }
            &server.launch();
        }
    }

    Ok(())
}
