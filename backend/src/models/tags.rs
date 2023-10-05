use crate::schema::*;
use diesel::{Identifiable, Queryable, Selectable};
use retronomicon_dto as dto;

#[derive(Clone, Debug, Queryable, Identifiable, Selectable)]
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
