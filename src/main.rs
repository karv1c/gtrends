use std::convert::Infallible;
use std::error::Error;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

mod handler;
mod trend_request;
use crate::handler::*;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let make_svc =
        make_service_fn(move |_conn| async move { Ok::<_, Infallible>(service_fn(handler)) });

    let addr = ([127, 0, 0, 1], 8000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}
