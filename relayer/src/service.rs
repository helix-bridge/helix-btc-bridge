mod btc;

// std
use std::{
	fmt::Debug,
	path::PathBuf,
	sync::Arc,
	thread::{self, ScopedJoinHandle},
	time::Duration,
};
// crates.io
use app_dirs2::AppDataType;
use deadpool_sqlite::Pool;
use tokio::runtime::Runtime;
// self
use crate::{conf::*, prelude::*, sql, APP_INFO};

trait Relay
where
	Self: Debug + Send + Sync,
{
	fn name(&self) -> &'static str;

	fn init(&self) -> Result<()>;

	fn run(&self) -> Result<()>;

	fn start(&self) -> Result<()> {
		self.init()?;
		self.run()?;

		Ok(())
	}
}

#[derive(Debug)]
struct Service {
	relayers: Vec<Box<dyn Relay>>,
}
impl Service {
	fn conf_path() -> Result<PathBuf> {
		Ok(app_dirs2::app_root(AppDataType::UserConfig, &APP_INFO)?.join("conf.toml"))
	}

	fn sql_path() -> Result<PathBuf> {
		Ok(app_dirs2::app_root(AppDataType::UserData, &APP_INFO)?.join("data.db3"))
	}

	fn register_context() -> Result<Context> {
		let rt = Runtime::new()?;
		let p = rt.block_on(async {
			let p = Self::sql_path()?;
			let p = sql::init(&p).await.map_err(|e| {
				tracing::error!(
					"an error occurred while initializing the database, please check {p:?}",
				);

				e
			})?;

			Ok::<_, Error>(p)
		})?;

		Ok(Context { runtime: Arc::new(rt), sql: Arc::new(p) })
	}

	fn register_relayers(context: Context) -> Result<Vec<Box<dyn Relay>>> {
		let p = Self::conf_path()?;
		let c = Conf::load_from(&p)?;
		let rs = vec![btc::Relayer::new(c.btc, context).map(|r| Box::new(r) as Box<dyn Relay>)]
			.into_iter()
			.collect::<Result<_>>()
			.map_err(|e| {
				tracing::error!(
					"an error occurred while parsing the configuration, please check {p:?}",
				);

				e
			})?;

		Ok(rs)
	}

	fn new() -> Result<Self> {
		let context = Self::register_context()?;
		let relayers = Self::register_relayers(context)?;

		Ok(Self { relayers })
	}
}

#[derive(Clone, Debug)]
struct Context {
	runtime: Arc<Runtime>,
	sql: Arc<Pool>,
}

pub fn run() -> Result<()> {
	let Service { relayers } = Service::new()?;

	thread::scope::<_, Result<()>>(|s| {
		let mut threads = relayers.iter().map(|r| Some(s.spawn(|| r.start()))).collect::<Vec<_>>();

		while !threads.is_empty() {
			let mut i = 0;

			while i < threads.len() {
				let r = &*relayers[i];
				let t = supervise(r, threads[i].take(), |r| s.spawn(|| r.start()))?;

				if t.is_some() {
					threads[i] = t;
					i += 1;
				} else {
					threads.remove(i);

					tracing::info!("{} service has completed", r.name());
				}
			}

			thread::sleep(Duration::from_millis(200));
		}

		Ok(())
	})?;

	Ok(())
}

fn supervise<'a, F>(
	relayer: &'a dyn Relay,
	thread: Option<ScopedJoinHandle<'a, Result<()>>>,
	restart_fn: F,
) -> Result<Option<ScopedJoinHandle<'a, Result<()>>>>
where
	F: FnOnce(&'a dyn Relay) -> ScopedJoinHandle<'a, Result<()>>,
{
	if let Some(t) = thread {
		if t.is_finished() {
			match t.join().map_err(Error::Any)? {
				Ok(_) => Ok(None),
				Err(e) => {
					tracing::error!("an error occurred while running {}: {e:?}", relayer.name());

					Ok(Some(restart_fn(relayer)))
				},
			}
		} else {
			Ok(Some(t))
		}
	} else {
		Ok(thread)
	}
}
