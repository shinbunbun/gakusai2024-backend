mod domain;
mod e2e_test;
mod error;
mod infrastructure;
mod interface;
mod usecase;
mod util;

use std::{env, sync::Arc};

use domain::repository::hello::HelloRepositoryTrait;
use dotenv::dotenv;
use sea_orm::Database;
use tokio::sync::Mutex;
use tonic::transport::Server;

use gakusai2024_proto::hello::hello_service_server::HelloServiceServer;
use interface::handler::hello::HelloHandlerTrait;
use usecase::hello::HelloUsecaseTrait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().ok();
    let addr = env::var("SERVER_ADDR")
        .expect("SERVER_ADDR must be set")
        .parse()?;
    let db_addr = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = Database::connect(db_addr).await?;

    // Dependency Injection
    let hello_persistence =
        infrastructure::db::hello::HelloPersistence::new(Arc::new(Mutex::new(conn)));
    let hello_usecase = usecase::hello::HelloUsecase::new(Box::new(hello_persistence));
    let hello_handler = interface::handler::hello::HelloHandler::new(Box::new(hello_usecase));

    log::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(HelloServiceServer::new(hello_handler))
        .serve(addr)
        .await?;

    Ok(())
}
