use std::{future::Future, sync::Arc};

use sea_orm::DatabaseConnection;

use crate::{domain::hello::Hello, error::CustomError};

pub trait HelloRepository {
    fn new(conn: Arc<DatabaseConnection>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello) -> impl Future<Output = Result<String, CustomError>> + Send;
    fn find(&self, name: String) -> impl Future<Output = Result<Hello, CustomError>> + Send;
}
