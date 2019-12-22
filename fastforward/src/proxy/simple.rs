// mod filters;

use futures::{Future, Stream};
use hyper::{Body, Client};
use hyper::client::{self, HttpConnector, Service};
use hyper::server::{self, Http, Request};
use hyper::Uri;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpStream;

use super::filters::filter_request_headers;
use super::setup_listener;
use super::generic_proxy;

struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub proxy_addr: SocketAddr,
}

pub fn simple_proxy(listen_addr: SocketAddr, proxy_addr: SocketAddr) {
   let pass_through_director: &'static _ = &move |req: &Request| {
       let base_url = format!(
           "http://{}{}",
           proxy_addr,
           req.uri().path(),
       );

       let url = match req.uri().query() {
           Some(qs) => format!("{}?{}", base_url, qs),
           None => base_url
       };

       println!("{}", url);
   };

   generic_proxy(listen_addr, pass_through_director);
}

// pub fn simple_proxy(listen_addr: SocketAddr, proxy_addr: SocketAddr) {
//     let mut core = Core::new().unwrap();
//     let handle = core.handle();
//     let listener = setup_listener(listen_addr, &handle).expect("Failed to setup listener");

//     let clients = listener.incoming();
//     let srv = clients.for_each(move |(socket, _)| {
//         _proxy(socket, proxy_addr, &handle);

//         Ok(())
//     });

//     core.run(srv).expect("Server failed");
// }

// fn _proxy(socket: TcpStream, addr: SocketAddr, handle: &Handle) {
//     socket.set_nodelay(true).unwrap();
//     let client = Client::configure()
//         // .connector(tm)
//         .build(&handle);

//     let service = Proxy {
//         client: client,
//         proxy_addr: addr,
//     };

//     println!("{}", addr);
//     let http: Http = Http::new();
//     let conn = http.serve_connection(socket, service);
//     let fut = conn.map_err(|e| eprintln!("server connection error: {}", e));

//     handle.spawn(fut);
// }

// impl Service for Proxy {
//     type Request = server::Request;
//     type Response = server::Response;
//     type Error = hyper::Error;
//     type Future = Box<Future<Item = server::Response, Error = Self::Error>>;

//     fn call(&self, req: server::Request) -> Self::Future {
//         println!("Method: {}", req.method());

//         let mut client_req = map_request(req);


//         let base_url = format!(
//             "http://{}{}",
//             self.proxy_addr,
//             client_req.uri().path(),
//             // client_req.uri().query().unwrap_or("")
//         );

//         let url = match client_req.uri().query() {
//             Some(qs) => format!("{}?{}", base_url, qs),
//             None => base_url
//         };

//         println!("{}", url);
//         let uri = Uri::from_str(&url).expect("Failed to parse url");
//         client_req.set_uri(uri);

//         // self.client.get(client_req.uri().clone());
//         let backend = self.client.call(client_req).then(move |resp| match resp {
//             Ok(resp) => {
//                 // debug!("Response: {}", res.status());
//                 // debug!("Headers: \n{}", res.headers());

//                 let server_response = map_response(resp);

//                 ::futures::finished(server_response)
//             }
//             Err(e) => {
//                 // error!("Error connecting to backend: {:?}", e);
//                 ::futures::failed(e)
//             }
//         });

//         Box::new(backend)
//     }
// }

// fn map_request(req: server::Request) -> client::Request {
//     let headers = filter_request_headers(req.headers());

//     // TODO fix clone
//     let mut r = client::Request::new(req.method().clone(), req.uri().clone());
//     r.headers_mut().extend(headers.iter());
//     r.set_body(req.body());
//     r
// }

// fn map_response(res: client::Response) -> server::Response {
//     let mut r = server::Response::new().with_status(res.status());

//     // let headers = filter_backend_response_headers(res.headers());
//     // r.headers_mut().extend(headers.iter());

//     r.set_body(res.body());
//     r
// }
