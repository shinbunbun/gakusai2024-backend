use std::{ops::Deref, sync::Arc};

use entity::task::{self, ActiveModel};
use sea_orm::{DatabaseConnection, EntityTrait, IntoSimpleExpr, QueryFilter, Set};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    domain::{repository::task::TaskRepositoryTrait, task::Task},
    error::CustomError,
};

use entity::task::Entity as TaskEntity;

use super::Repository;

pub struct TaskPersistence {
    repository: Repository,
}

impl TaskRepositoryTrait for TaskPersistence {
    fn new(conn: Arc<Mutex<DatabaseConnection>>) -> Self
    where
        Self: Sized,
    {
        Self {
            repository: Repository::new(conn),
        }
    }
    async fn insert(&self, task: Task) -> Result<Uuid, CustomError> {
        let db_unlock = self.repository.get_db();
        let db_lock = db_unlock.lock().await;
        let db = db_lock.deref();
        let task_am = ActiveModel {
            id: Set(task.id),
            title: Set(task.title),
            description: Set(task.description),
            due_date: Set(task.due_date),
            priority: Set(task.priority),
            weight: Set(task.weight),
            created_at: Set(task.created_at),
            updated_at: Set(task.updated_at),
            user_id: Set(task.user_id),
        };
        let insert_result = TaskEntity::insert(task_am).exec(db).await?;
        Ok(insert_result.last_insert_id)
    }

    async fn find(&self, id: Uuid) -> Result<Task, CustomError> {
        let db_unlock = self.repository.get_db();
        let db_lock = db_unlock.lock().await;
        let db = db_lock.deref();
        let result = TaskEntity::find()
            .filter(task::Column::Id.into_simple_expr().eq(id))
            .one(db)
            .await?;
        match result {
            Some(task) => Ok(Task {
                id: task.id,
                title: task.title,
                description: task.description,
                due_date: task.due_date,
                priority: task.priority,
                weight: task.weight,
                created_at: task.created_at,
                updated_at: task.updated_at,
                user_id: task.user_id,
            }),
            None => Err(CustomError::DbNotFound(format!("key: {}", &id))),
        }
    }

    async fn find_from_user_id(&self, user_id: String) -> Result<Vec<Task>, CustomError> {
        let db_unlock = self.repository.get_db();
        let db_lock = db_unlock.lock().await;
        let db = db_lock.deref();

        let result = TaskEntity::find()
            .filter(task::Column::UserId.into_simple_expr().eq(&user_id))
            .all(db)
            .await?;

        Ok(result
            .iter()
            .map(|t| Task {
                id: t.id,
                title: t.title.clone(),
                description: t.description.clone(),
                due_date: t.due_date,
                priority: t.priority,
                weight: t.weight,
                created_at: t.created_at,
                updated_at: t.updated_at,
                user_id: t.user_id.clone(),
            })
            .collect())
    }
}
