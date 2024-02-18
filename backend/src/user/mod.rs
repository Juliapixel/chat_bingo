use log::error;
use sqlx::{prelude::FromRow, Executor, Postgres};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub twitch_id: String,
    pub twitch_login: String,
    pub twitch_display_name: String,
    pub twitch_token: Option<String>
}

impl User {
    pub fn new(
        user_id: impl Into<Uuid>,
        twitch_id: impl Into<String>,
        twitch_login: impl Into<String>,
        twitch_display_name: impl Into<String>,
        twitch_token: impl Into<String>
    ) -> Self {
        Self {
            user_id: user_id.into(),
            twitch_id: twitch_id.into(),
            twitch_login: twitch_login.into(),
            twitch_display_name: twitch_display_name.into(),
            twitch_token: Some(twitch_token.into())
        }
    }

    /// performs an UPSERT on the database and updates `self` with the correct UUID in case the user already existed
    pub async fn upsert<'a, E>(&mut self, conn: E) -> Result<(), sqlx::Error>
    where E: Executor<'a, Database = Postgres>
    {
        match sqlx::query_as::<E::Database, User>("INSERT INTO users (user_id, twitch_id, twitch_login, twitch_display_name, twitch_token)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (twitch_id) DO UPDATE
            SET twitch_login = $3,
            twitch_display_name = $4,
            twitch_token = $5
            RETURNING user_id, twitch_id, twitch_login, twitch_display_name, twitch_token;"
        ).bind(self.user_id)
            .bind(&self.twitch_id)
            .bind(&self.twitch_login)
            .bind(&self.twitch_display_name)
            .bind(&self.twitch_token)
            .fetch_one(conn).await {
                Ok(o) => {
                    *self = o;
                    Ok(())
                },
                Err(e) => Err(e),
            }
    }

    /// tries to get a [User] from the database via their ULID
    pub async fn get_from_ulid<'a, E>(conn: E, ulid: Ulid) -> Option<Self>
    where E: Executor<'a, Database = Postgres>
    {
        let resp = sqlx::query_as::<E::Database, User>("SELECT
        (user_id, twitch_id, twitch_login, twitch_display_name, twitch_token)
        FROM users WHERE
        user_id = $1;").bind(Uuid::from(ulid)).fetch_one(conn).await;

        match resp {
            Ok(o) => Some(o),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => {
                error!("{e}");
                None
            }
        }
    }

    /// tries to get a [User] from the database via their twitch ID
    pub async fn get_from_twitch_id<'a, E>(conn: E, id: &str) -> Option<Self>
    where E: Executor<'a, Database = Postgres>
    {
        let resp = sqlx::query_as::<E::Database, User>("SELECT
        (user_id, twitch_id, twitch_login, twitch_display_name, twitch_token)
        FROM users WHERE
        twitch_id = $1;").bind(id).fetch_one(conn).await;

        match resp {
            Ok(o) => Some(o),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => {
                error!("{e}");
                None
            }
        }
    }

}
