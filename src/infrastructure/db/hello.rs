use std::sync::Arc;

use entity::hello::{self, ActiveModel};
use sea_orm::{DatabaseConnection, EntityTrait, IntoSimpleExpr, QueryFilter, Set};

use crate::domain::repository::hello::HelloRepository;

use entity::hello::Entity as HelloEntity;

use super::Repository;

pub struct HelloPersistence {
    repository: Repository,
}

impl HelloRepository for HelloPersistence {
    fn new(conn: Arc<DatabaseConnection>) -> Self
    where
        Self: Sized,
    {
        Self {
            repository: Repository::new(conn),
        }
    }
    async fn insert(&self, hello: crate::domain::hello::Hello) {
        let db = &*self.repository.get_db();
        let hello_am = ActiveModel {
            name: Set(hello.name),
            message: Set(hello.message),
        };
        let _ = HelloEntity::insert(hello_am).exec(db).await.unwrap();
    }

    async fn find(&self, name: String) -> crate::domain::hello::Hello {
        let db = &*self.repository.get_db();
        HelloEntity::find()
            .filter(hello::Column::Name.into_simple_expr().eq(name))
            .one(db)
            .await
            .unwrap()
            .unwrap()
    }
}
