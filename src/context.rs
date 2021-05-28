use crate::config::Config;
use crate::processor::Processor;
use lever::sync::atomics::AtomicBox;
use std::sync::Arc;
use http_types::{Body, Request, Response, Error};
use crate::errors::*;
use isahc::{http as ih, RequestExt, ResponseExt, HttpClient};
use isahc::{Body as IHBody};
use std::io::{BufRead, Read};
use futures::io::BufReader;
use nuclei::{spawn_blocking, spawn};

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    processors: Arc<AtomicBox<Vec<Arc<dyn Processor>>>>,
    client: HttpClient
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            processors: Arc::new(AtomicBox::new(vec![])),
            client: HttpClient::new().unwrap()
        }
    }

    pub fn add_processor(&self, processor: Box<dyn Processor>) {
        let appender = &[processor.into()];
        self.processors.replace_with(|e| {
            [e.as_slice(), &appender[..]].concat()
        });
    }

    pub async fn traverse(&self, request: Request) -> Result<Response> {
        let mut final_req = self.processors.get().iter().try_fold(request, |mut acc, e| -> Result<Request> {
            acc = e.pre_request(acc)?;
            Ok(acc)
        })?;

        let response = self.forward(final_req).await?;

        let final_res = self.processors.get().iter().rev().try_fold(response, |mut acc, e| -> Result<Response> {
            acc = e.post_response(acc)?;
            Ok(acc)
        })?;

        Ok(final_res)
    }

    async fn forward(&self, mut request: Request) -> Result<Response> {
        let mut builder = ih::Request::builder()
            .uri(request.url().as_str())
            .method(ih::Method::from_bytes(request.method().to_string().as_bytes())?);

        for (name, value) in request.iter() {
            builder = builder.header(name.as_str(), value.as_str());
        }

        let body: Body = request.take_body();
        let body: IHBody = match body.len() {
            Some(len) => IHBody::from_reader_sized(body, len as u64),
            None => IHBody::from_reader(body),
        };

        let request = builder.body(body)?;

        let res = self.client.send_async(request).await?;

        let maybe_metrics = res.metrics().cloned();
        let (parts, body) = res.into_parts();
        let body = Body::from_reader(BufReader::new(body), None);
        let mut response = http_types::Response::new(parts.status.as_u16());
        for (name, value) in &parts.headers {
            response.append_header(name.as_str(), value.to_str().unwrap());
        }

        if let Some(metrics) = maybe_metrics {
            response.ext_mut().insert(metrics);
        }

        response.set_body(body);
        Ok(response)
    }
}