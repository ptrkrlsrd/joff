# Joff
![](https://camo.githubusercontent.com/0b67f2eb691b83144519058d27f3ae6104f24a760db25d4a0566c7c40f53731f/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f727573742d2532333030303030302e7376673f7374796c653d666f722d7468652d6261646765266c6f676f3d72757374266c6f676f436f6c6f723d7768697465)
[![Rust](https://github.com/ptrkrlsrd/joff/actions/workflows/rust.yml/badge.svg)](https://github.com/ptrkrlsrd/joff/actions/workflows/rust.yml)
## Store JSON responses locally and serve them locally using Rocket

Since this project is using Rocket.rs, you'll have to use the nightly toolchain by running: `rustup override set nightly`.

## Installation
* Run:
    * `rustup default nightly`
    * `cargo install joff`

## Usage
```
joff 1.0

USAGE:
    joff [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bucket-name <bucket-name>    Name of the KV bucket [default: json_data]
    -d, --data-path <data-path>        Path to the KV store [default: ./data]

SUBCOMMANDS:
    add      
    help     Prints this message or the help of the given subcommand(s)
    list     
    serve 
```
