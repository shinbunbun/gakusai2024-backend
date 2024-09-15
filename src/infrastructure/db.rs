use std::sync::Arc;

use sea_orm::DatabaseConnection;

pub mod hello;

struct Repository {
    db: Arc<DatabaseConnection>,
}

impl Repository {
    fn new(conn: Arc<DatabaseConnection>) -> Self {
        Self { db: conn }
    }
    fn get_db(&self) -> Arc<DatabaseConnection> {
        self.db.clone()
    }
}
