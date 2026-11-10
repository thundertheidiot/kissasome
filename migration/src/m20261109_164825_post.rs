use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261109_164825_post"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("posts")
                    .if_not_exists()
                    .col(pk_auto("id").auto_increment().not_null())
                    .col(integer("user_id").not_null())
                    .col(uuid_uniq("image").not_null())
                    .col(string("text").not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_user")
                            .from("posts", "user_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("posts").to_owned())
            .await
    }
}
