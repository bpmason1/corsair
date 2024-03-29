extern crate clap;
extern crate fastforward;
extern crate http;
extern crate once_cell;
extern crate term;

use clap::{Arg, Command};
use fastforward::generic_proxy;
use http::{
    header::HeaderValue,
    Method,
    Response,
    StatusCode
};
use once_cell::sync::Lazy;
use std::net::SocketAddr;


fn req_transform(req: &mut http::Request<Vec<u8>>) -> Option<Response<Vec<u8>>> { 
    match req.method() {
        &Method::OPTIONS => {
            // println!("{:?}", req);

            let body = Vec::new();
            let mut resp = Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .body(body).unwrap();

            let resp_headers = resp.headers_mut();
            let allowed_addresses = HeaderValue::from_str("*").unwrap();
            let allowed_headers = HeaderValue::from_str("*").unwrap();
            let allowed_methods = HeaderValue::from_str("GET, POST, PATCH, PUT, DELETE, OPTIONS").unwrap();
            resp_headers.insert(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, allowed_addresses);
            resp_headers.insert(http::header::ACCESS_CONTROL_ALLOW_HEADERS, allowed_headers);
            resp_headers.insert(http::header::ACCESS_CONTROL_ALLOW_METHODS, allowed_methods);
            // println!("{:?}", resp);
            Some(resp)
        },
        _ => {
            println!("This was not an option request");
            let proxy_addr_str = MATCHES.value_of("proxy-ip").expect("proxy-ip could not be read");

            let proxy_addr = HeaderValue::from_str(proxy_addr_str).unwrap();

            let req_headers = req.headers_mut();
            req_headers.remove(http::header::HOST);
            req_headers.insert(http::header::HOST, proxy_addr);

            None
        }
    }
}

fn resp_transform(resp: &mut http::Response<Vec<u8>>) { 
    let allowed_addresses = HeaderValue::from_str("*").unwrap();
    let allowed_headers = HeaderValue::from_str("*").unwrap();
    let allowed_methods = HeaderValue::from_str("GET, POST, PATCH, PUT, DELETE, OPTIONS").unwrap();

    let resp_headers = resp.headers_mut();
    resp_headers.insert(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, allowed_addresses);
    resp_headers.insert(http::header::ACCESS_CONTROL_ALLOW_HEADERS, allowed_headers);
    resp_headers.insert(http::header::ACCESS_CONTROL_ALLOW_METHODS, allowed_methods);
}

static MATCHES : Lazy<clap::ArgMatches> = Lazy::new( || {
  get_command_line_matches()
});

fn main() {
    let mut terminal = term::stdout().unwrap();

    let listen_ip_str = MATCHES.value_of("listen-ip").expect("listen-ip could not be read");
    let listen_ip = listen_ip_str.parse::<SocketAddr>().unwrap();

    // This is to ensure that the "proxy-ip" parameter was passed in
    match MATCHES.value_of("proxy-ip") {
        Some(_) => {
            generic_proxy(listen_ip, req_transform, resp_transform)
        },
        None => {
            terminal.fg(term::color::RED).unwrap();
            println!("ERROR: proxy-ip could not be read");
            terminal.reset().unwrap();
        }
    };

    
}

fn get_command_line_matches() -> clap::ArgMatches {
    return Command::new("corsair")
        .arg(
            Arg::with_name("listen-ip")
                .long("listen-ip")
                .value_name("listen-ip")
                .takes_value(true)
                .required(true)
                .help(
                    "address where the application listens for incoming messages. example: 0.0.0.0:8000",
                ),
        )
        .arg(
            Arg::with_name("proxy-ip")
                .long("proxy-ip")
                .value_name("proxy-ip")
                .takes_value(true)
                .required(true)
                .help(
                    "address where the application proxies incoming messages. example: 127.0.0.1:8888",
                ),
        )
        .arg(
            Arg::with_name("permissive")
                .long("permissive")
                .value_name("permissive")
                .takes_value(false)
                .required(true)
                .help(
                    "if permissive is true then allow CORS for all requests/domains",
                ),
        )
        .get_matches();
}
