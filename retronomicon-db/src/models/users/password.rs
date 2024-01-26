use crate::models::User;
use crate::schema::user_passwords::dsl;
use crate::{schema, Db};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{OptionalExtension, QueryDsl};
use rand::Rng;
use rocket::error;
use rocket_db_pools::diesel::RunQueryDsl;

pub struct DbPassword(String);

impl From<&UserPassword> for DbPassword {
    fn from(user_password: &UserPassword) -> Self {
        Self(user_password.password.clone())
    }
}

impl From<DbPassword> for String {
    fn from(db_password: DbPassword) -> Self {
        db_password.0
    }
}

impl DbPassword {
    fn argon(pepper: &[u8]) -> Result<Argon2, anyhow::Error> {
        Argon2::new_with_secret(
            pepper,
            Algorithm::default(),
            Version::default(),
            Params::default(),
        )
        .map_err(|e| anyhow::Error::msg(e.to_string()))
    }

    pub fn create(pepper: &[u8], password: &str) -> Result<Self, anyhow::Error> {
        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        let password_hash = Self::argon(pepper)?
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?
            .to_string();

        Ok(Self(password_hash))
    }

    pub fn verify(&self, pepper: &[u8], password: &str) -> Result<bool, anyhow::Error> {
        let parsed_hash =
            PasswordHash::new(&self.0).map_err(|e| anyhow::Error::msg(e.to_string()))?;
        Ok(Self::argon(pepper)?
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Identifiable)]
#[diesel(primary_key(user_id))]
#[diesel(belongs_to(User))]
#[diesel(table_name = schema::user_passwords)]
pub struct UserPassword {
    pub user_id: i32,
    pub password: String,
    pub updated_at: NaiveDateTime,
    pub needs_reset: bool,
    pub validation_token: Option<String>,
}

impl UserPassword {
    async fn from_user(db: &mut Db, user: &User) -> Result<Option<Self>, diesel::result::Error> {
        schema::user_passwords::table
            .filter(schema::user_passwords::user_id.eq(user.id))
            .first::<Self>(db)
            .await
            .optional()
    }

    pub async fn from_validation_token(
        db: &mut Db,
        email: &str,
        token: &str,
    ) -> Result<Option<(User, Self)>, diesel::result::Error> {
        schema::user_passwords::table
            .inner_join(schema::users::table)
            .filter(schema::users::email.eq(email))
            .filter(schema::user_passwords::validation_token.eq(token))
            .select((
                schema::users::all_columns,
                schema::user_passwords::all_columns,
            ))
            .first(db)
            .await
            .optional()
    }

    pub async fn create(
        db: &mut Db,
        user: &User,
        password: Option<&str>,
        pepper: &[u8],
        create_token: bool,
    ) -> Result<Self, anyhow::Error> {
        let needs_reset = password.is_none() || password == Some("");
        let password_hash: String = if let Some(password) = password {
            DbPassword::create(pepper, password)?.into()
        } else {
            "".to_string()
        };

        let validation_token = if create_token {
            let mut buffer = [0; 32];
            rand::thread_rng().fill(&mut buffer);
            Some(URL_SAFE_NO_PAD.encode(&buffer))
        } else {
            None
        };

        Ok(diesel::insert_into(schema::user_passwords::table)
            .values((
                schema::user_passwords::user_id.eq(user.id),
                schema::user_passwords::password.eq(&password_hash),
                schema::user_passwords::updated_at.eq(chrono::Utc::now().naive_utc()),
                schema::user_passwords::needs_reset.eq(needs_reset),
                schema::user_passwords::validation_token.eq(validation_token),
            ))
            .returning(schema::user_passwords::all_columns)
            .get_result(db)
            .await?)
    }

    pub async fn verify_password(
        db: &mut Db,
        user: User,
        password: &str,
        pepper: &[u8],
    ) -> Result<Option<(User, Self)>, diesel::result::Error> {
        let user_password = match Self::from_user(db, &user).await? {
            Some(user_password) => user_password,
            None => return Ok(None),
        };
        if user_password.password == "" {
            return Ok(None);
        }

        let pass = DbPassword::from(&user_password);
        match pass.verify(pepper, password) {
            Ok(true) => Ok(Some((user, user_password))),
            Ok(false) => Ok(None),
            Err(e) => {
                error!("Failed to verify password: {}", e);
                Ok(None)
            }
        }
    }

    pub async fn validated(&self, db: &mut Db) -> Result<(), diesel::result::Error> {
        diesel::update(dsl::user_passwords.filter(dsl::user_id.eq(self.user_id)))
            .set((dsl::validation_token.eq::<Option<String>>(None),))
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, db: &mut Db) -> Result<(), diesel::result::Error> {
        diesel::delete(dsl::user_passwords.filter(dsl::user_id.eq(self.user_id)))
            .execute(db)
            .await?;
        Ok(())
    }
}

#[test]
fn can_hash_password() {
    let password = "password";
    let pepper = "pepper";
    let password_hash = DbPassword::create(pepper.as_bytes(), password).unwrap();
    eprintln!("password_hash: {}", password_hash.0);
    assert!(password_hash.verify(pepper.as_bytes(), password).unwrap());
    assert!(!password_hash.verify(pepper.as_bytes(), "wrong").unwrap());
}
