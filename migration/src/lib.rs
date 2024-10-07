pub use sea_orm_migration::prelude::*;

mod m20240915_062259_create_hello_table;
mod m20241004_014513_create_task_table;
mod m20241004_030615_create_user_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240915_062259_create_hello_table::Migration),
            Box::new(m20241004_014513_create_task_table::Migration),
            Box::new(m20241004_030615_create_user_table::Migration),
        ]
    }
}
