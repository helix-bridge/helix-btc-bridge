// std
use std::path::Path;
// crates.io
use chrono::{DateTime, Utc};
use deadpool_sqlite::{Config, Object, Pool, Runtime::Tokio1};
use once_cell::sync::OnceCell;
use rusqlite::{
	types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
	Connection,
};
use uuid::Uuid;
// self
use crate::{prelude::*, x::*};

static SQL: OnceCell<Pool> = OnceCell::new();

pub trait Sql
where
	Self: X,
{
	async fn acquire_sql() -> Result<Object> {
		Ok(unsafe { SQL.get_unchecked() }.get().await.map_err(DeadpoolSqliteError::Pool)?)
	}

	async fn interact<F>(f: F) -> Result<usize>
	where
		F: 'static + Send + FnOnce(&Connection) -> rusqlite::Result<usize>,
	{
		Self::acquire_sql()
			.await?
			.interact(|sql| Ok(f(sql)?))
			.await
			.map_err(DeadpoolSqliteError::Interact)?
	}

	async fn get(&self) -> Result<XRecord> {
		Self::acquire_sql().await?;

		todo!()
	}

	async fn insert(&self, record: XRecord) -> Result<usize> {
		Self::interact(move |sql| {
			sql.execute(
				"INSERT OR REPLACE INTO x_records (\
					id,\
					source,\
					sender,\
					target,\
					recipient,\
					amount,\
					hash,\
					created_at,\
					finished_at\
				) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
				rusqlite::params![
					record.id,
					record.source,
					record.sender,
					record.target,
					record.recipient,
					record.amount.to_string(),
					record.hash,
					record.created_at.to_rfc3339(),
					record.finished_at.map(|d| d.to_rfc3339())
				],
			)
		})
		.await
	}
}
impl<T> Sql for T where T: X {}

#[derive(Debug)]
pub struct XRecord {
	pub id: Uuid,
	pub source: Id,
	pub sender: String,
	pub target: Id,
	pub recipient: String,
	pub amount: u128,
	pub hash: Option<String>,
	pub created_at: DateTime<Utc>,
	pub finished_at: Option<DateTime<Utc>>,
}

impl FromSql for Id {
	fn column_result(value: ValueRef) -> FromSqlResult<Self> {
		Ok(Self(value.as_i64()? as _))
	}
}
impl ToSql for Id {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
		Ok(ToSqlOutput::from(self.0))
	}
}

pub async fn init<P>(path: P) -> Result<()>
where
	P: AsRef<Path>,
{
	let pool =
		Config::new(path.as_ref()).create_pool(Tokio1).map_err(DeadpoolSqliteError::Create)?;

	let sql = pool.get().await.map_err(DeadpoolSqliteError::Pool)?;

	sql.interact(|sql| {
		sql.execute(
			"CREATE TABLE IF NOT EXISTS x_records (\
				id BLOB PRIMARY KEY,\
				source INTEGER NOT NULL,\
				sender TEXT NOT NULL,\
				target INTEGER NOT NULL,\
				recipient TEXT NOT NULL,\
				amount TEXT NOT NULL,\
				hash TEXT,\
				created_at TEXT NOT NULL,\
				finished_at TEXT\
			)",
			[],
		)
	})
	.await
	.map_err(DeadpoolSqliteError::Interact)??;

	SQL.set(pool).unwrap();

	Ok(())
}