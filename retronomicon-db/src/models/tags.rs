use crate::types::FromIdOrSlug;
use crate::{models, schema};
use diesel::internal::operators_macro::FieldAliasMapper;
use diesel::prelude::*;
use diesel::{Identifiable, Queryable, Selectable};
use retronomicon_dto as dto;
use rocket_db_pools::diesel::{AsyncConnection, RunQueryDsl};

#[derive(Clone, Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = schema::tags)]
pub struct Tag {
    pub id: i32,
    pub slug: String,
    pub description: Option<String>,
    pub color: i64,
}

impl From<Tag> for dto::tags::Tag {
    fn from(value: Tag) -> Self {
        dto::tags::Tag {
            id: value.id,
            slug: value.slug,
            color: value.color as u32,
        }
    }
}

#[rocket::async_trait]
impl FromIdOrSlug for Tag {
    async fn from_id(db: &mut crate::Db, id: i32) -> Result<Option<Self>, diesel::result::Error>
    where
        Self: Sized,
    {
        schema::tags::table
            .filter(schema::tags::id.eq(id))
            .first::<Self>(db)
            .await
            .optional()
    }

    async fn from_slug(
        db: &mut crate::Db,
        slug: &str,
    ) -> Result<Option<Self>, diesel::result::Error>
    where
        Self: Sized,
    {
        schema::tags::table
            .filter(schema::tags::slug.eq(slug))
            .first::<Self>(db)
            .await
            .optional()
    }
}

impl Tag {
    pub async fn create(
        db: &mut crate::Db,
        slug: String,
        description: String,
        color: u32,
    ) -> Result<Self, diesel::result::Error> {
        diesel::insert_into(schema::tags::table)
            .values((
                schema::tags::slug.eq(slug),
                schema::tags::description.eq(description),
                schema::tags::color.eq((color & 0x00FFFFFF) as i64),
            ))
            .returning(schema::tags::all_columns)
            .get_result::<Self>(db)
            .await
    }

    pub async fn delete(&self, db: &mut crate::Db) -> Result<(), diesel::result::Error> {
        diesel::delete(schema::tags::table.filter(schema::tags::id.eq(self.id)))
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn list(
        db: &mut crate::Db,
        page: i64,
        limit: i64,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        schema::tags::table
            .offset(page * limit)
            .limit(limit)
            .load::<Self>(db)
            .await
    }
}
