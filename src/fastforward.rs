use hyper::Body;
use hyper::Client;
use hyper::client::HttpConnector;
use std::net::SocketAddr;


pub struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub proxy_addr: SocketAddr,
}
