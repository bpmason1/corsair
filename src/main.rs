#[macro_use]
extern crate hyper;

extern crate clap;
extern crate futures;
extern crate log;
extern crate net2;
extern crate tokio_core;


use clap::{Arg, App};
use futures::{Future, Stream};
use hyper::{Body, Client, Headers};
use hyper::client::{self, HttpConnector, Service};
use hyper::header;
use hyper::server::{self, Http};
use hyper::Uri;
use net2::TcpBuilder;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::{TcpListener, TcpStream};


header! { (KeepAlive, "Keep-Alive") => [String] }

fn main() {
    let matches = get_command_line_matches();
    let listen_ip_str = matches.value_of("listen-ip").expect("listen-ip could not be read");
    let listen_ip = listen_ip_str.parse::<SocketAddr>().unwrap();

    let proxy_ip_str = matches.value_of("proxy-ip").expect("proxy-ip could not be read");
    let proxy_ip = proxy_ip_str.parse::<SocketAddr>().unwrap();

    println!("{}", listen_ip_str);
    println!("{}", proxy_ip_str);

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let listener = setup_listener(listen_ip, &handle).expect("Failed to setup listener");

    let clients = listener.incoming();
    let srv = clients.for_each(move |(socket, _)| {
        proxy(socket, proxy_ip, &handle);

        Ok(())
    });

    core.run(srv).expect("Server failed");
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

fn setup_listener(addr: SocketAddr, handle: &Handle) -> io::Result<TcpListener> {
    let listener = TcpBuilder::new_v4()?;
    // listener.reuse_address(true)?;
    // listener.reuse_port(true)?;
    let listener = listener.bind(&addr)?;
    let listener = listener.listen(128)?;
    let listener = TcpListener::from_listener(listener, &addr, &handle)?;

    Ok(listener)
}

fn map_request(req: server::Request) -> client::Request {
    let headers = filter_frontend_request_headers(req.headers());

    // TODO fix clone
    let mut r = client::Request::new(req.method().clone(), req.uri().clone());
    r.headers_mut().extend(headers.iter());
    r.set_body(req.body());
    r
}

fn map_response(res: client::Response) -> server::Response {
    let mut r = server::Response::new().with_status(res.status());

    // let headers = filter_backend_response_headers(res.headers());
    // r.headers_mut().extend(headers.iter());

    r.set_body(res.body());
    r
}

struct Proxy {
    // client: Client<TimeoutConnector<HttpsConnector<HttpConnector>>, Body>,
    client: Client<HttpConnector, Body>,
    proxy_addr: std::net::SocketAddr,
}

impl Service for Proxy {
    type Request = server::Request;
    type Response = server::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = server::Response, Error = Self::Error>>;

    fn call(&self, req: server::Request) -> Self::Future {
        println!("Method: {}", req.method());

        let mut client_req = map_request(req);


        let url = format!(
            "http://{}{}?{}",
            self.proxy_addr,
            client_req.uri().path(),
            client_req.uri().query().unwrap_or("")
        );

        println!("{}", url);
        let uri = Uri::from_str(&url).expect("Failed to parse url");
        client_req.set_uri(uri);

        // self.client.get(client_req.uri().clone());
        let backend = self.client.call(client_req).then(move |resp| match resp {
            Ok(resp) => {
                // debug!("Response: {}", res.status());
                // debug!("Headers: \n{}", res.headers());

                let server_response = map_response(resp);

                ::futures::finished(server_response)
            }
            Err(e) => {
                // error!("Error connecting to backend: {:?}", e);
                ::futures::failed(e)
            }
        });

        Box::new(backend)
    }
}

fn proxy(socket: TcpStream, addr: SocketAddr, handle: &Handle) {
    socket.set_nodelay(true).unwrap();
    let client = Client::configure()
        // .connector(tm)
        .build(&handle);

    let service = Proxy {
        client: client,
        proxy_addr: addr,
    };

    println!("{}", addr);
    let http = Http::new();
    http.bind_connection(&handle, socket, addr, service);
}

pub fn filter_frontend_request_headers(headers: &Headers) -> Headers {

    let mut h = headers.clone();
    headers.get::<header::Connection>().and_then(|c| {
        for c_h in &c.0 {

            match c_h {
                &header::ConnectionOption::Close => {
                    let _ = h.remove_raw("Close");
                }

                &header::ConnectionOption::KeepAlive => {
                    let _ = h.remove::<KeepAlive>(); //_raw("Keep-Alive");
                }

                &header::ConnectionOption::ConnectionHeader(ref o) => {
                    let _ = h.remove_raw(&o);
                }
            }
        }

        Some(())
    });

    let _ = h.remove::<header::Connection>();
    let _ = h.remove::<header::TransferEncoding>();
    let _ = h.remove::<header::Upgrade>();

    h
}


#[test]
/// Per RFC 2616 Section 13.5.1 - MUST remove hop-by-hop headers
/// Per RFC 7230 Section 6.1 - MUST remove Connection and Connection option headers
fn test_filter_frontend_request_headers() {
    // defining these here only to let me assert
    header! { (Foo, "Foo") => [String] }
    header! { (Bar, "Bar") => [String] }

    let header_vec = vec![
        ("TE", "gzip"),
        ("Transfer-Encoding", "chunked"),
        ("Host", "example.net"),
        ("Connection", "Keep-Alive, Foo"),
        ("Bar", "abc"),
        ("Foo", "def"),
        ("Keep-Alive", "timeout=30"),
        ("Upgrade", "HTTP/2.0, SHTTP/1.3, IRC/6.9, RTA/x11"),
    ];

    let mut headers = Headers::new();

    for (name, value) in header_vec {
        headers.set_raw(name, value);
    }

    let given = filter_frontend_request_headers(&headers);

    assert_eq!(false, given.has::<header::TransferEncoding>());
    assert_eq!(true, given.has::<header::Host>());
    assert_eq!(false, given.has::<header::Connection>());
    assert_eq!(false, given.has::<Foo>());
    assert_eq!(true, given.has::<Bar>());
    assert_eq!(false, given.has::<KeepAlive>());
    assert_eq!(false, given.has::<header::Upgrade>());
}
