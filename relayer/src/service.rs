mod btc;

// std
use std::{fmt::Debug, path::PathBuf, sync::Arc, time::Duration};
// crates.io
use app_dirs2::{AppDataType, AppInfo};
use deadpool_sqlite::Pool;
use tokio::{
	runtime::{Builder, Runtime},
	task, time,
};
// self
use crate::{conf::*, prelude::*, sql};

const APP_INFO: AppInfo = AppInfo { name: "helix-btc-bridge-relayer", author: "Xavier Lau" };

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
	context: Context,
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
		let rt = Builder::new_multi_thread()
			.enable_all()
			// TODO: Need more tests.
			// Increare this if there is a new relayer.
			.worker_threads(1)
			.build()?;
		let p = Self::sql_path()?;
		let p = sql::init(&p).map_err(|e| {
			tracing::error!(
				"an error occurred while initializing the database, please check {p:?}",
			);

			e
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
		let relayers = Self::register_relayers(context.clone())?;

		Ok(Self { context, relayers })
	}
}

#[derive(Clone, Debug)]
struct Context {
	runtime: Arc<Runtime>,
	sql: Arc<Pool>,
}

pub fn run() -> Result<()> {
	let Service { context, relayers } = Service::new()?;

	context.runtime.block_on(async {
		let mut tasks = relayers
			.iter()
			.map(|r| {
				let r = unsafe { &*(&**r as *const dyn Relay) };

				task::spawn(async { r.start() })
			})
			.collect::<Vec<_>>();

		loop {
			let mut finished = 0;

			for i in 0..tasks.len() {
				let r = unsafe { &*(&*relayers[i] as *const dyn Relay) };
				let t = &mut tasks[i];

				if t.is_finished() {
					match t.await? {
						Ok(_) => {
							tracing::info!("{} service has completed", r.name());

							finished += 1;
						},
						Err(e) => {
							tracing::error!("an error occurred while running {}: {e:?}", r.name());

							tasks[i] = task::spawn(async { r.start() });
						},
					}
				}
			}

			if finished == tasks.len() {
				break;
			}

			time::sleep(Duration::from_millis(5_000)).await;
		}

		// Terminate all async tasks as we are about to shut down the tokio runtime reactor here.
		context.sql.close();

		Ok(())
	})
}
