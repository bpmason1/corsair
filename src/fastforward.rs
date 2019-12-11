use futures::Future;
use hyper::Body;
use hyper::Client;
use hyper::client::{self, HttpConnector, Service};
use hyper::header;
use hyper::Headers;
use hyper::server::{self, Http};
use hyper::Uri;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::reactor::Handle;
use tokio_core::net::TcpStream;


header! { (KeepAlive, "Keep-Alive") => [String] }

struct Proxy {
    pub client: Client<HttpConnector, Body>,
    pub proxy_addr: SocketAddr,
}


pub fn proxy(socket: TcpStream, addr: SocketAddr, handle: &Handle) {
    socket.set_nodelay(true).unwrap();
    let client = Client::configure()
        // .connector(tm)
        .build(&handle);

    let service = Proxy {
        client: client,
        proxy_addr: addr,
    };

    println!("{}", addr);
    let http: Http = Http::new();
    let conn = http.serve_connection(socket, service);
    let fut = conn.map_err(|e| eprintln!("server connection error: {}", e));

    handle.spawn(fut);
}

impl Service for Proxy {
    type Request = server::Request;
    type Response = server::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = server::Response, Error = Self::Error>>;

    fn call(&self, req: server::Request) -> Self::Future {
        println!("Method: {}", req.method());

        let mut client_req = map_request(req);


        let base_url = format!(
            "http://{}{}",
            self.proxy_addr,
            client_req.uri().path(),
            // client_req.uri().query().unwrap_or("")
        );

        let url = match client_req.uri().query() {
            Some(qs) => format!("{}?{}", base_url, qs),
            None => base_url
        };

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


fn filter_frontend_request_headers(headers: &Headers) -> Headers {

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
