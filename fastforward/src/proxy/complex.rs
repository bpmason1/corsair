use futures::{Future, Stream};
use hyper::{Body, Client};
use hyper::client::{HttpConnector, Service};
use hyper::server::{self, Http, Request};
use std::net::SocketAddr;
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Handle};

use super::setup_listener;
use super::filters::filter_request_headers;

type Director = Fn(& Request);
// trait Director {
//     pub fn call(&mut Request);
// }

struct Proxy<'p> {
    pub client: Client<HttpConnector, Body>,
    pub director: &'p Director,
}

// trait ProxyRule {}

// fn pass_through_director(req: &mut Request) {
// }

// pub fn pass_through_proxy(listen_addr: SocketAddr) {
//     let director = pass_through_director;
//     generic_proxy(listen_addr, director)
// }

pub fn generic_proxy(listen_addr: SocketAddr, director: &'static Director)
 {
    let mut core = Core::new().unwrap();
    let handle: Handle = core.handle();
    let listener = setup_listener(listen_addr, &handle).expect("Failed to setup listener");

    let clients = listener.incoming();
    let srv = clients.for_each(move |(socket, _)| {
        _proxy(socket, &handle, director);
        Ok(())
    });

    core.run(srv).expect("Server failed");
}

fn _proxy(socket: TcpStream, handle: &Handle, director: &'static Director) {
    socket.set_nodelay(true).unwrap();
    let client = Client::configure()
        // .connector(tm)
        .build(&handle);

    let service = Proxy {
        client: client,
        director: director,
    };

    // // println!("{}", addr);
    let http: Http = Http::new();
    let conn = http.serve_connection(socket, service);
    let fut = conn.map_err(|e| eprintln!("server connection error: {}", e));

    // handle.spawn(fut);
}

impl<'s> Service for Proxy<'s> {
    type Request = server::Request;
    type Response = server::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = server::Response, Error = Self::Error>>;

    fn call<'p>(&self, mut req: server::Request) -> Self::Future {
        println!("Method: {}", req.method());

        let headers = filter_request_headers(req.headers());
        req.headers_mut().clear();
        req.headers_mut().extend(headers.iter());

        (self.director)(&mut req);
        println!("{}", req.uri());

        let backend = self.client.call(req).then(move |resp| match resp {
            Ok(resp) => {
                ::futures::finished(resp)
            }
            Err(e) => {
                ::futures::failed(e)
            }
        });

        Box::new(backend)
    }
}
