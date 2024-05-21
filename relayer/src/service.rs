mod btc;

// std
use std::{fmt::Debug, thread, time::Duration};
// self
use crate::{conf::Conf, prelude::*};

trait Relayer
where
	Self: Debug + Send + Sync,
{
	fn name(&self) -> &'static str;

	fn init(&self) -> Result<()> {
		Ok(())
	}

	fn run(&self) -> Result<()>;

	fn start(&self) -> Result<()> {
		self.init()?;
		self.run()?;

		Ok(())
	}
}

#[derive(Debug)]
struct RelayerPool {
	relayers: Vec<Box<dyn Relayer>>,
}
impl RelayerPool {
	fn load() -> Result<Self> {
		let p = Conf::default_path()?;

		match Conf::load_from(&p)?.try_into() {
			Ok(r) => Ok(r),
			r => {
				tracing::error!(
					"an error occurred while parsing the configuration, \
					please check the {p:?}",
				);

				r
			},
		}
	}
}
impl TryFrom<Conf> for RelayerPool {
	type Error = Error;

	fn try_from(value: Conf) -> Result<Self> {
		Ok(Self { relayers: vec![Box::new(btc::Relayer::try_from(value.btc)?)] })
	}
}

pub fn run() -> Result<()> {
	let pool = RelayerPool::load()?;

	thread::scope::<_, Result<()>>(|s| {
		let mut handles =
			pool.relayers.iter().map(|r| Some(s.spawn(move || r.start()))).collect::<Vec<_>>();

		loop {
			for (i, maybe_h) in handles.iter_mut().enumerate() {
				let Some(h) = maybe_h else {
					continue;
				};

				if h.is_finished() {
					let h = maybe_h.take().unwrap();

					if let Err(e) = h.join().map_err(Error::Any)? {
						let r = &pool.relayers[i];

						tracing::error!("an error occurred while running {}: {e:?}", r.name());

						*maybe_h = Some(s.spawn(|| r.start()));
					}
				}
			}

			thread::sleep(Duration::from_millis(200));
		}
	})?;

	Ok(())
}
