use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::domain::hello::Hello;

pub trait HelloRepository: Send + Sync {
    fn new(conn: Arc<DatabaseConnection>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello);
    fn find(&self, name: String) -> Hello;
}
