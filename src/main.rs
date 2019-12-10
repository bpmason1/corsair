extern crate clap;

use clap::{Arg, App};

fn main() {
    let matches = get_command_line_matches();
    let listen_ip_str = matches.value_of("listen-ip").expect("listen-ip could not be read");
    let proxy_ip_str = matches.value_of("proxy-ip").expect("proxy-ip could not be read");

    println!("{}", listen_ip_str);
    println!("{}", proxy_ip_str);
}

fn get_command_line_matches() -> clap::ArgMatches<'static> {
    return App::new("corsair")
        .arg(
            Arg::with_name("listen-ip")
                .long("listen-ip")
                .value_name("listen-ip")
                .takes_value(true)
                .help(
                    "address where the application listens for incoming messages. example: 0.0.0.0:8687",
                ),
        )
        .arg(
            Arg::with_name("proxy-ip")
                .long("proxy-ip")
                .value_name("proxy-ip")
                .takes_value(true)
                .help(
                    "address where the application proxies incoming messages. example: 0.0.0.0:8687",
                ),
        )
        .get_matches();
}
