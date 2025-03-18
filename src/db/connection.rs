use std::{path::Path, time::Duration};
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use crate::error::Error;

pub async fn new_connection<P: AsRef<Path>>(db_name: P) -> Result<SqlitePool, Error>
{
    let local_path = Path::new(&std::env::current_dir().unwrap()).join([db_name.as_ref().to_str().unwrap(), ".", "sq3"].concat());
    if !local_path.exists()
    {
        std::fs::File::create(&local_path)?;
    }
    let options = SqliteConnectOptions::new()
    .filename(local_path)
    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect_with(options)
    .await?;
    Ok(pool)
}