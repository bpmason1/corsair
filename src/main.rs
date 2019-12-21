#[macro_use]
extern crate hyper;

extern crate clap;
extern crate futures;
extern crate log;
extern crate net2;
extern crate tokio_core;

mod ff;

use clap::{Arg, App};
use ff::proxy;
use std::net::SocketAddr;


fn main() {
    let matches = get_command_line_matches();
    let listen_ip_str = matches.value_of("listen-ip").expect("listen-ip could not be read");
    let listen_ip = listen_ip_str.parse::<SocketAddr>().unwrap();

    let proxy_ip_str = matches.value_of("proxy-ip").expect("proxy-ip could not be read");
    let proxy_ip = proxy_ip_str.parse::<SocketAddr>().unwrap();

    proxy(listen_ip, proxy_ip)
}

fn get_command_line_matches() -> clap::ArgMatches<'static> {
    return App::new("corsair")
        .arg(
            Arg::with_name("listen-ip")
                .long("listen-ip")
                .value_name("listen-ip")
                .takes_value(true)
                .help(
                    "address where the application listens for incoming messages. example: 0.0.0.0:8000",
                ),
        )
        .arg(
            Arg::with_name("proxy-ip")
                .long("proxy-ip")
                .value_name("proxy-ip")
                .takes_value(true)
                .help(
                    "address where the application proxies incoming messages. example: 127.0.0.1:8888",
                ),
        )
        .get_matches();
}
