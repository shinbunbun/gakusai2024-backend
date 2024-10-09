use entity::task::{Column as TaskColumn, Entity as Task};
use entity::user::{Column as UserColumn, Entity as User};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create_foreign_key()はentityにRelationを書いた場合必要ない？
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
        // Replace the sample below with your own migration scripts
        todo!();

        manager
            .drop_table(Table::drop().table(User).to_owned())
            .await
    }
}
