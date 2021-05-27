mod error;
mod config;
mod processor;

use tracing::*;
use tracing_subscriber::FmtSubscriber;

use nuclei::*;
use std::net::TcpListener;

use clap::*;
use anyhow::Result;
use async_dup::Arc as ADArc;
use futures::prelude::*;
use http_types::{Request, Response, StatusCode};
use lever::prelude::LOTable;

use crate::config::Config;

/// Serves a request and returns a response.
async fn serve(req: Request, config: Config) -> http_types::Result<Response> {
    println!("Serving {}", req.url());

    let mut res = Response::new(StatusCode::Ok);
    res.insert_header("Content-Type", "text/plain");
    res.set_body("Hello from async-h1!");
    Ok(res)
}

/// Listens for incoming connections and serves them.
async fn listen(listener: Handle<TcpListener>, config: Config) -> Result<()> {
    // Format the full host address.
    let host = format!("http://{}", listener.get_ref().local_addr()?);
    println!("Listening on {}", host);

    loop {
        // Accept the next connection.
        let (stream, _) = listener.accept().await?;

        // Spawn a background task serving this connection.
        let stream = ADArc::new(stream);
        let rtc = config.clone();
        spawn(async move {
            if let Err(err) = async_h1::accept(stream, |req| serve(req, rtc.clone())).await {
                println!("Connection error: {:#?}", err);
            }
        });
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    spawn_blocking(|| drive(future::pending::<()>()));

    let matches = App::new("relay")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Relay - Reverse proxy server")
        .arg(
            Arg::with_name("debug")
                .help("turn on debugging information")
                .short("d"),
        )
        .args(&[
            Arg::with_name("config")
                .help("sets the config file to use")
                .takes_value(true)
                .short("c")
                .long("config"),
        ])
        .get_matches();

    let config = matches.value_of("config")
        .map_or_else(|| {
            info!("Config wasn't given. Falling back to defaults.");
            Config::default()
        }, |config_file| {
            Config::default()
                .with_config_file(config_file)
                .build_with_config_file()
                .expect("Config file is not readable.")
        });

    block_on(async {
        let http =
            listen(Handle::<TcpListener>::bind(config.host_port())?, config);

        http.await;
        Ok(())
    })
}
