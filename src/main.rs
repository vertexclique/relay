mod error;
mod config;

use nuclei::*;
use std::net::TcpListener;

use anyhow::Result;
use async_dup::Arc as ADArc;
use futures::prelude::*;
use http_types::{Request, Response, StatusCode};
use lever::prelude::LOTable;

/// Serves a request and returns a response.
async fn serve(req: Request, rt: LOTable<String, String>) -> http_types::Result<Response> {
    println!("Serving {}", req.url());

    let mut res = Response::new(StatusCode::Ok);
    res.insert_header("Content-Type", "text/plain");
    res.set_body("Hello from async-h1!");
    Ok(res)
}

/// Listens for incoming connections and serves them.
async fn listen(listener: Handle<TcpListener>, rt: LOTable<String, String>) -> Result<()> {
    // Format the full host address.
    let host = format!("http://{}", listener.get_ref().local_addr()?);
    println!("Listening on {}", host);

    loop {
        // Accept the next connection.
        let (stream, _) = listener.accept().await?;

        // Spawn a background task serving this connection.
        let stream = ADArc::new(stream);
        let rtc = rt.clone();
        spawn(async move {
            if let Err(err) = async_h1::accept(stream, |req| serve(req, rtc.clone())).await {
                println!("Connection error: {:#?}", err);
            }
        });
    }
}

fn main() -> Result<()> {
    spawn_blocking(|| drive(future::pending::<()>()));

    let routing_table: LOTable<String, String> = LOTable::new();

    block_on(async {
        let http = listen(Handle::<TcpListener>::bind("0.0.0.0:8000")?, routing_table);

        http.await;
        Ok(())
    })
}
