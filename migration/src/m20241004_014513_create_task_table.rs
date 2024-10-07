use sea_orm_migration::prelude::*;
use entity::task::{Column as TaskColumn, Entity as Task};
use entity::user::{Column as UserColumn, Entity as User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Task)
                    .if_not_exists()
                    .col(ColumnDef::new(TaskColumn::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(TaskColumn::Title).string().not_null())
                    .col(ColumnDef::new(TaskColumn::Description).string().not_null())
                    .col(ColumnDef::new(TaskColumn::DueDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TaskColumn::Priority).integer().not_null())
                    .col(ColumnDef::new(TaskColumn::Weight).integer().not_null())
                    .col(ColumnDef::new(TaskColumn::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TaskColumn::UpdatedAt).timestamp_with_time_zone().not_null())
                    // create_foreign_key()を使うならこの行は必要ない？
                    .col(ColumnDef::new(TaskColumn::UserId).string().not_null())
                    .to_owned(),
            )
            .await?;

        // create_foreign_key()はentityにRelationを書いた場合必要ない？
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                .name("FK_User_Id")
                .from(Task, TaskColumn::UserId)
                .to(User, UserColumn::Id)
                .on_delete(ForeignKeyAction::Restrict)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Task).to_owned())
            .await
    }
}
