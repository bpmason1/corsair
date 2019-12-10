extern crate clap;
extern crate futures;
extern crate log;
extern crate net2;
extern crate tokio_core;


use clap::{Arg, App};
use futures::stream::Stream;
use net2::TcpBuilder;
use std::io;
use std::net::SocketAddr;
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::{TcpListener, TcpStream};

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
    let srv = clients.for_each(move |(socket, addr)| {
        proxy(socket, addr, &handle);

        Ok(())
    });

    core.run(srv).expect("Server failed");
}

fn get_command_line_matches() -> clap::ArgMatches<'static> {
    return App::new("weldr")
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

fn proxy(socket: TcpStream, addr: SocketAddr, handle: &Handle) {
    println!("Proxying");
    socket.set_nodelay(true).unwrap();
}
