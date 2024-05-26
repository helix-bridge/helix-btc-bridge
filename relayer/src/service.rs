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
use tokio::runtime::Runtime;
// self
use crate::{conf::*, prelude::*, sql, APP_INFO};

trait Relay
where
	Self: Debug + Send + Sync,
{
	fn name(&self) -> &'static str;

	// fn init(&self) -> Result<()> {
	// 	Ok(())
	// }

	fn run(&self, runtime: Arc<Runtime>) -> Result<()>;

	fn start(&self, runtime: Arc<Runtime>) -> Result<()> {
		// self.init()?;
		self.run(runtime)?;

		Ok(())
	}
}

#[derive(Debug)]
struct Service {
	runtime: Arc<Runtime>,
	relayers: Vec<Box<dyn Relay>>,
}
impl Service {
	fn conf_path() -> Result<PathBuf> {
		Ok(app_dirs2::app_root(AppDataType::UserConfig, &APP_INFO)?.join("conf.toml"))
	}

	fn sql_path() -> Result<PathBuf> {
		Ok(app_dirs2::app_root(AppDataType::UserData, &APP_INFO)?.join("data.db3"))
	}

	fn register_components() -> Result<Runtime> {
		let rt = Runtime::new()?;
		let p = Self::sql_path()?;

		rt.block_on(async {
			sql::init(&p).await.map_err(|e| {
				tracing::error!(
					"an error occurred while initializing the database, please check {p:?}",
				);

				e
			})
		})?;

		Ok(rt)
	}

	fn register_relayers() -> Result<Vec<Box<dyn Relay>>> {
		let p = Self::conf_path()?;
		let c = Conf::load_from(&p)?;
		let rs = vec![btc::Relayer::try_from(c.btc).map(|r| Box::new(r) as Box<dyn Relay>)]
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

	fn init() -> Result<Self> {
		let rt = Self::register_components()?;
		let relayers = Self::register_relayers()?;

		Ok(Self { runtime: Arc::new(rt), relayers })
	}
}

pub fn run() -> Result<()> {
	let Service { runtime, relayers } = Service::init()?;

	thread::scope::<_, Result<()>>(|s| {
		let mut threads =
			relayers.iter().map(|r| Some(s.spawn(|| r.start(runtime.clone())))).collect::<Vec<_>>();

		while !threads.is_empty() {
			let mut i = 0;

			while i < threads.len() {
				let r = &*relayers[i];
				let t = supervise(r, threads[i].take(), |r| s.spawn(|| r.start(runtime.clone())))?;

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
