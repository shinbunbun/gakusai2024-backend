use std::sync::Arc;

use entity::hello::{self, ActiveModel};
use sea_orm::{DatabaseConnection, EntityTrait, IntoSimpleExpr, QueryFilter, Set};

use crate::{
    domain::{hello::Hello, repository::hello::HelloRepository},
    error::CustomError,
};

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
    async fn insert(&self, hello: Hello) -> Result<String, CustomError> {
        let db = &*self.repository.get_db();
        let hello_am = ActiveModel {
            name: Set(hello.name),
            message: Set(hello.message),
        };
        let insert_result = HelloEntity::insert(hello_am).exec(db).await?;
        Ok(insert_result.last_insert_id.to_string())
    }

    async fn find(&self, name: String) -> Result<Hello, CustomError> {
        let db = &*self.repository.get_db();
        let result = HelloEntity::find()
            .filter(hello::Column::Name.into_simple_expr().eq(&name))
            .one(db)
            .await?;
        match result {
            Some(hello) => Ok(Hello {
                name: hello.name,
                message: hello.message,
            }),
            None => Err(CustomError::DbNotFound(format!("key: {}", &name))),
        }
    }
}
