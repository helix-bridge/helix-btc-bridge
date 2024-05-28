// std
use std::{path::Path, sync::Arc};
// crates.io
use chrono::{DateTime, Utc};
use deadpool_sqlite::{Config, Object, Pool, Runtime::Tokio1};
use rusqlite::{
	types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
	Connection, OptionalExtension,
};
// self
use crate::{prelude::*, x::*};

pub trait Sql
where
	Self: X,
{
	async fn pool(&self) -> &Arc<Pool>;

	async fn sql(&self) -> Result<Object> {
		Ok(self.pool().await.get().await.map_err(DeadpoolSqliteError::Pool)?)
	}

	async fn interact<F, T>(&self, f: F) -> Result<T>
	where
		F: 'static + Send + FnOnce(&Connection) -> Result<T>,
		T: 'static + Send,
	{
		self.sql().await?.interact(|sql| f(sql)).await.map_err(DeadpoolSqliteError::Interact)?
	}

	async fn init(&self) -> Result<()> {
		self.interact(move |sql| {
			sql.execute(
				&format!(
					"CREATE TABLE IF NOT EXISTS [{}] (\
					id INTEGER PRIMARY KEY AUTOINCREMENT,\
					block_height INTEGER NOT NULL,\
					txid TEXT NOT NULL,\
					target INTEGER NOT NULL,\
					recipient TEXT NOT NULL,\
					amount TEXT NOT NULL,\
					hash TEXT,\
					created_at DATETIME NOT NULL,\
					finished_at DATETIME\
				)",
					Self::NAME
				),
				(),
			)?;

			Ok(())
		})
		.await
	}

	async fn get_latest(&self) -> Result<Option<XRecord>> {
		let Some((block_height, txid, target, recipient, amount, hash, created_at, finished_at)) =
			self.interact(move |sql| {
				let mut stmt = sql
					.prepare(
						&format!("SELECT * FROM [{}] ORDER BY id DESC LIMIT 1", Self::NAME,),
					)?;
				let maybe_xr = stmt
					.query_row((), |r| {
						Ok((
							r.get(1)?,
							r.get(2)?,
							r.get(3)?,
							r.get(4)?,
							r.get::<_, String>(5)?,
							r.get(6)?,
							r.get(7)?,
							r.get(8)?,
						))
					})
					.optional()?;

				Ok(maybe_xr)
			})
			.await?
		else {
			return Ok(None);
		};
		let xr = XRecord {
			block_height,
			txid,
			target,
			recipient,
			amount: amount.parse()?,
			hash,
			created_at,
			finished_at,
		};

		Ok(Some(xr))
	}

	async fn insert(&self, records: Vec<XRecord>) -> Result<()> {
		self.interact(move |c| {
			let sql = format!(
				"INSERT OR REPLACE INTO [{}] (\
				block_height,\
				txid,\
				target,\
				recipient,\
				amount,\
				hash,\
				created_at,\
				finished_at\
			) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
				Self::NAME
			);

			records.into_iter().try_for_each(|r| {
				c.execute(
					&sql,
					rusqlite::params![
						r.block_height,
						r.txid,
						r.target,
						r.recipient,
						r.amount.to_string(),
						r.hash,
						r.created_at,
						r.finished_at
					],
				)?;

				Ok(())
			})
		})
		.await
	}
}

#[derive(Debug)]
pub struct XRecord {
	pub block_height: u64,
	pub txid: String,
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

pub fn init<P>(path: P) -> Result<Pool>
where
	P: AsRef<Path>,
{
	Ok(Config::new(path.as_ref()).create_pool(Tokio1).map_err(DeadpoolSqliteError::Create)?)
}
