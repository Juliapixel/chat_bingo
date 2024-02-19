use chrono::{DateTime, Utc};
use log::error;
use sqlx::{postgres::PgArguments, prelude::FromRow, query::{Query, QueryAs}, Executor, Postgres};
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub twitch_id: String,
    pub twitch_login: String,
    pub twitch_display_name: String,
}

impl User {
    pub fn new(
        user_id: impl Into<Uuid>,
        twitch_id: impl Into<String>,
        twitch_login: impl Into<String>,
        twitch_display_name: impl Into<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            twitch_id: twitch_id.into(),
            twitch_login: twitch_login.into(),
            twitch_display_name: twitch_display_name.into(),
        }
    }

    /// performs an UPSERT on the database and updates `self` with the correct UUID in case the user already existed
    pub async fn upsert<'a, E>(&mut self, conn: E) -> Result<(), sqlx::Error>
    where E: Executor<'a, Database = Postgres>
    {
        match sqlx::query_as::<E::Database, User>("INSERT INTO users (user_id, twitch_id, twitch_login, twitch_display_name)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (twitch_id) DO UPDATE
            SET twitch_login = $3,
            twitch_display_name = $4
            RETURNING user_id, twitch_id, twitch_login, twitch_display_name;"
        ).bind(self.user_id)
            .bind(&self.twitch_id)
            .bind(&self.twitch_login)
            .bind(&self.twitch_display_name)
            .fetch_one(conn).await {
                Ok(o) => {
                    *self = o;
                    Ok(())
                },
                Err(e) => Err(e),
            }
    }

    /// tries to get a [User] from the database via their ULID
    pub fn get_from_ulid(ulid: Ulid) -> QueryAs<'static, Postgres, User, PgArguments>
    {
        sqlx::query_as::<Postgres, User>("SELECT
        (user_id, twitch_id, twitch_login, twitch_display_name, twitch_token)
        FROM users WHERE
        user_id = $1;").bind(Uuid::from(ulid))
    }

    /// tries to get a [User] from the database via their twitch ID
    pub fn get_from_twitch_id(id: &str) -> QueryAs<'_, Postgres, User, PgArguments> {
        sqlx::query_as::<Postgres, User>("SELECT
        (user_id, twitch_id, twitch_login, twitch_display_name, twitch_token)
        FROM users WHERE
        twitch_id = $1;").bind(id)
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TwitchToken {
    pub token: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: String
}

impl TwitchToken {
    pub fn new(
        token: impl Into<String>,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        refresh_token: impl Into<String>
    ) -> Self {
        Self {
            token: token.into(),
            issued_at,
            expires_at,
            refresh_token: refresh_token.into()
        }
    }

    pub fn get_from_user_ulid<'a>(ulid: Ulid) -> QueryAs<'a, Postgres, Self, PgArguments> {
        sqlx::query_as::<Postgres, Self>("SELECT (
            twitch_tokens.token,
            twitch_tokens.issued_at,
            twitch_tokens.expires_at,
            twitch_tokens.refresh_token
        ) FROM twitch_tokens
        INNER JOIN users ON twitch_tokens.user_id = users.id
        WHERE users.user_id=$1").bind(Uuid::from(ulid))
    }

    pub fn upsert_for_ulid(&self, ulid: Ulid) -> Query<'_, Postgres, PgArguments> {
        sqlx::query::<Postgres>("INSERT INTO twitch_tokens (
            user_id,
            token,
            issued_at,
            expires_at,
            refresh_token
        ) SELECT users.id, $1, $2, $3, $4
        FROM users
        WHERE users.user_id = $5
        ON CONFLICT (user_id)
        DO UPDATE
        SET token = $1,
        issued_at = $2,
        expires_at = $3,
        refresh_token = $4")
            .bind(&self.token)
            .bind(self.issued_at)
            .bind(self.expires_at)
            .bind(&self.refresh_token)
            .bind(Uuid::from(ulid))
    }
}
