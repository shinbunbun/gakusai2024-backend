use std::{env, sync::Arc};

use dotenv::dotenv;
use gakusai2024_backend::domain::repository::hello::HelloRepositoryTrait;
use gakusai2024_backend::domain::repository::task::TaskRepositoryTrait;
use gakusai2024_proto::api::hello_service_server::HelloServiceServer;
use gakusai2024_proto::api::task_service_server::TaskServiceServer;
use sea_orm::Database;
use tokio::sync::Mutex;
use tonic::transport::Server;

use gakusai2024_backend::infrastructure;
use gakusai2024_backend::interface;
use gakusai2024_backend::interface::handler::hello::HelloHandlerTrait;
use gakusai2024_backend::interface::handler::task::TaskHandlerTrait;
use gakusai2024_backend::usecase;
use gakusai2024_backend::usecase::hello::HelloUsecaseTrait;
use gakusai2024_backend::usecase::task::TaskUsecaseTrait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().ok();
    let addr = env::var("SERVER_ADDR")
        .expect("SERVER_ADDR must be set")
        .parse()?;
    let db_addr = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = Arc::new(Mutex::new(Database::connect(db_addr).await?));

    // Dependency Injection
    let hello_persistence = infrastructure::db::hello::HelloPersistence::new(conn.clone());
    let hello_usecase = usecase::hello::HelloUsecase::new(Box::new(hello_persistence));
    let hello_handler = interface::handler::hello::HelloHandler::new(Box::new(hello_usecase));

    let task_persistence = infrastructure::db::task::TaskPersistence::new(conn);
    let task_usecase = usecase::task::TaskUsecase::new(Box::new(task_persistence));
    let task_handler = interface::handler::task::TaskHandler::new(Box::new(task_usecase));

    log::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(HelloServiceServer::new(hello_handler))
        .add_service(TaskServiceServer::new(task_handler))
        .serve(addr)
        .await?;

    Ok(())
}
