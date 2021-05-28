use http_types::{Request, Response};
use crate::errors::RelayError;
use crate::errors::*;

pub trait Processor: Send + Sync {
    fn pre_request(&self, request: Request) -> Result<Request>;
    fn post_response(&self, response: Response) -> Result<Response>;
}