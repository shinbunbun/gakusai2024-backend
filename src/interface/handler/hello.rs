use gakusai2024_proto::hello::{hello_service_server::HelloService, HelloRequest, HelloResponse};
use tonic::{Request, Response, Status};

use crate::{domain::hello::Hello, usecase::hello::HelloUsecaseTrait};

pub trait HelloHandlerTrait {
    fn new(usecase: Box<dyn HelloUsecaseTrait>) -> Self
    where
        Self: Sized;
}

pub struct HelloHandler {
    usecase: Box<dyn HelloUsecaseTrait>,
}

impl HelloHandlerTrait for HelloHandler {
    fn new(usecase: Box<dyn HelloUsecaseTrait>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl HelloService for HelloHandler {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let reply = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}
