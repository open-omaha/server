//! SQLite storage for application metadata.

use open_omaha_domain::{Application, CupPolicy, VersionScheme};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions};

pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let options: SqliteConnectOptions = url.parse()?;
        let pool = SqlitePoolOptions::new()
            // A single persistent connection also keeps in-memory databases alive.
            .max_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options.create_if_missing(true).foreign_keys(true))
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!().run(&self.pool).await
    }

    pub async fn create_application(&self, app: &Application) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO applications (id, omaha_app_id, display_name, version_scheme, cup_policy) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&app.id)
        .bind(&app.omaha_app_id)
        .bind(&app.display_name)
        .bind(app.version_scheme.name())
        .bind(app.cup_policy.name())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_application(&self, id: &str) -> Result<Option<Application>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, omaha_app_id, display_name, version_scheme, cup_policy \
             FROM applications WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| {
            Ok(Application {
                id: row.try_get("id")?,
                omaha_app_id: row.try_get("omaha_app_id")?,
                display_name: row.try_get("display_name")?,
                version_scheme: VersionScheme::from_name(row.try_get("version_scheme")?)
                    .ok_or_else(|| decode_error("version_scheme"))?,
                cup_policy: CupPolicy::from_name(row.try_get("cup_policy")?)
                    .ok_or_else(|| decode_error("cup_policy"))?,
            })
        })
        .transpose()
    }
}

fn decode_error(column: &str) -> sqlx::Error {
    sqlx::Error::ColumnDecode {
        index: column.into(),
        source: "unrecognized application setting".into(),
    }
}
