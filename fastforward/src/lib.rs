#[macro_use]
extern crate hyper;

extern crate futures;
extern crate log;
extern crate net2;
extern crate tokio_core;

mod proxy;

pub use proxy::simple_proxy;
pub use proxy::generic_proxy;