use std::{future::Future, sync::Arc};

use crate::{domain::task::Task, error::CustomError};
use mockall::automock;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[automock]
pub trait TaskRepositoryTrait {
    fn new(conn: Arc<tokio::sync::Mutex<DatabaseConnection>>) -> Self
    where
        Self: Sized;
    fn insert(&self, task: Task) -> impl Future<Output = Result<Uuid, CustomError>> + Send;
    fn find(&self, id: Uuid) -> impl Future<Output = Result<Task, CustomError>> + Send;
}
