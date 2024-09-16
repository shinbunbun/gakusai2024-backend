use std::{env, sync::Arc};

use dotenv::dotenv;
use gakusai2024_backend::domain::repository::hello::HelloRepositoryTrait;
use gakusai2024_proto::api::hello_service_server::HelloServiceServer;
use sea_orm::Database;
use tokio::sync::Mutex;
use tonic::transport::Server;

use gakusai2024_backend::infrastructure;
use gakusai2024_backend::interface;
use gakusai2024_backend::interface::handler::hello::HelloHandlerTrait;
use gakusai2024_backend::usecase;
use gakusai2024_backend::usecase::hello::HelloUsecaseTrait;

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
