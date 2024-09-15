mod domain;
mod infrastructure;
mod interface;
mod usecase;

use std::sync::Arc;

use domain::repository::repository::HelloRepository;
use sea_orm::Database;
use tonic::transport::Server;

use gakusai2024_proto::hello::hello_service_server::HelloServiceServer;
use interface::handler::hello::HelloHandlerTrait;
use usecase::hello::HelloUsecaseTrait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let addr = "127.0.0.1:50051".parse()?;

    let conn = Database::connect("postgres://postgres:postgrespassword@localhost:5432").await?;

    // Dependency Injection
    let hello_persistence = infrastructure::db::hello::HelloPersistence::new(Arc::new(conn));
    let hello_usecase = usecase::hello::HelloUsecase::new(Box::new(hello_persistence));
    let hello_handler = interface::handler::hello::HelloHandler::new(Box::new(hello_usecase));

    log::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(HelloServiceServer::new(hello_handler))
        .serve(addr)
        .await?;

    Ok(())
}
