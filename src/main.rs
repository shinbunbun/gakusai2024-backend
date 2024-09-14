use tonic::{transport::Server, Request, Response, Status};

use gakusai2024_proto::hello::hello_service_server::{HelloService, HelloServiceServer};
use gakusai2024_proto::hello::{HelloRequest, HelloResponse};

#[derive(Debug, Default)]
pub struct MyHello {}

#[tonic::async_trait]
impl HelloService for MyHello {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let addr = "127.0.0.1:50051".parse()?;
    let greeter = MyHello::default();

    log::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(HelloServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
