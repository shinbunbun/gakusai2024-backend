use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

pub mod hello;

struct Repository {
    db: Arc<Mutex<DatabaseConnection>>,
}

impl Repository {
    fn new(conn: Arc<Mutex<DatabaseConnection>>) -> Self {
        Self { db: conn }
    }
    fn get_db(&self) -> Arc<Mutex<DatabaseConnection>> {
        self.db.clone()
    }
}
