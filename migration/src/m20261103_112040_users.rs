use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261103_112040_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.create_table(
			Table::create()
				.table("users")
				.if_not_exists()
				.col(pk_auto("id"))
				.col(string("username").unique_key())
				.col(string("password_hash"))
				.to_owned()
		).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.drop_table(Table::drop().table("users").to_owned()).await
    }
}
