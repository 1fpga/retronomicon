use crate::db::Db;
use crate::error::Error;
use crate::{models, schema, user};
use retronomicon_dto as dto;
use retronomicon_dto::user::{UserDetails, UserUpdate};
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route};
use rocket_db_pools::diesel::prelude::*;
use scoped_futures::ScopedFutureExt;
use serde_json::Value;
use std::collections::BTreeMap;

#[get("/me/check?<username>")]
async fn me_check(mut db: Db, username: String) -> Result<Json<bool>, Error> {
    let exists = schema::users::table
        .filter(schema::users::username.eq(&Some(username)))
        .first::<models::User>(&mut db)
        .await
        .is_ok();

    Ok(Json(exists))
}

#[post("/me/update", format = "application/json", data = "<form>")]
async fn me_update(
    mut db: Db,
    cookies: &CookieJar<'_>,
    mut user: user::User,
    form: Json<UserUpdate<'_>>,
) -> Result<Json<dto::Ok>, Error> {
    match (user.username, form.username) {
        (Some(_), Some(_)) => Err(Error::Request("Username already set".into())),
        (None, None) => Err(Error::Request(
            "Username must be set on first update".into(),
        )),
        (None, Some(_)) | (Some(_), None) => Ok(()),
    }?;

    #[derive(AsChangeset)]
    #[diesel(table_name = schema::users)]
    struct UserSignupChangeset<'a> {
        username: Option<&'a str>,
        display_name: Option<&'a str>,
        description: Option<&'a str>,
        links: Option<Value>,
        metadata: Option<Value>,
    }

    let username = form.username.map(|x| x.to_string());

    db.transaction(|db| {
        async move {
            let mut changeset = UserSignupChangeset {
                username: form.username,
                display_name: form.display_name,
                description: form.description,
                links: None,
                metadata: None,
            };

            if let Some(links) = form.links.as_ref() {
                changeset.links = Some(serde_json::to_value(links).unwrap());
            } else {
                let mut links = BTreeMap::new();
                if !form.add_links.is_empty() || !form.remove_links.is_empty() {
                    let user: models::User = schema::users::table.find(user.id).first(db).await?;

                    if let Some(Value::Object(user_links)) = user.links {
                        for (k, v) in user_links.into_iter() {
                            if v.is_string() {
                                links.insert(k.to_string(), v);
                            }
                        }
                    }

                    for (k, v) in form.add_links.iter() {
                        links.insert(k.to_string(), serde_json::to_value(v).unwrap());
                    }
                    for k in form.remove_links.iter() {
                        links.remove(&k.to_string());
                    }
                }
                changeset.links = Some(serde_json::to_value(links).unwrap());
            }

            diesel::update(schema::users::table)
                .filter(schema::users::id.eq(user.id))
                .set(changeset)
                .execute(db)
                .await?;

            Result::<(), Error>::Ok(())
        }
        .scope_boxed()
    })
    .await?;

    // At this point, because of the unique constraint on username, we know
    // that the username is set.
    user.username = username.map(|x| x.to_string());
    user.update_cookie(cookies);

    Ok(Json(dto::Ok))
}

#[get("/me")]
async fn me(mut db: Db, user: user::User) -> Result<Json<UserDetails>, Error> {
    schema::users::table
        .filter(schema::users::id.eq(user.id))
        .first::<models::User>(&mut db)
        .await
        .map_err(|e| e.into())
        .map(|user| UserDetails {
            user: dto::user::UserDetailsInner {
                id: user.id,
                username: user.username.unwrap_or_default(),
                description: user.description,
                links: user.links,
                metadata: user.metadata,
            },
            groups: vec![],
        })
        .map(Json)
}

pub fn routes() -> Vec<Route> {
    routes![me, me_check, me_update,]
}
