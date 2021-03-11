# Joff
[![Rust](https://github.com/ptrkrlsrd/joff/actions/workflows/rust.yml/badge.svg)](https://github.com/ptrkrlsrd/joff/actions/workflows/rust.yml)
## Store JSON responses locally and serve them locally using Rocket. Note: This is the Rust version of my Golang tool [Acache](https://github.com/ptrkrlsrd/acache).

Since this project is using Rocket.rs, you'll have to use the nightly toolchain by running: `rustup override set nightly`.



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