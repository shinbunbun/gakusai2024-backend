use entity::task::{Column as TaskColumn, Entity as Task};
use entity::user::{Column as UserColumn, Entity as User};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("FK_User_Id")
                    .from(Task, TaskColumn::UserId)
                    .to(User, UserColumn::Id)
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("FK_User_Id")
                    .table(Task)
                    .to_owned(),
            )
            .await
    }
}
