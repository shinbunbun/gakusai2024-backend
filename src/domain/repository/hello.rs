use std::{future::Future, sync::Arc};

use sea_orm::DatabaseConnection;

use crate::domain::hello::Hello;

pub trait HelloRepository {
    fn new(conn: Arc<DatabaseConnection>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello) -> impl Future<Output = ()> + Send;
    fn find(&self, name: String) -> impl Future<Output = Hello> + Send;
}
