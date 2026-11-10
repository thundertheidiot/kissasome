pub use sea_orm_migration::prelude::*;

mod m20261103_112040_user;
mod m20261109_164825_post;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261103_112040_user::Migration),
            Box::new(m20261109_164825_post::Migration),
        ]
    }
}
