mod btc;

// std
use std::{
	fmt::Debug,
	thread::{self, ScopedJoinHandle},
	time::Duration,
};
// self
use crate::{conf::*, prelude::*};

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
	btc_relayer: btc::Relayer,
	other_relayers: Vec<Box<dyn Relayer>>,
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
		Ok(Self { btc_relayer: btc::Relayer::try_from(value.btc)?, other_relayers: Vec::new() })
	}
}

pub fn run() -> Result<()> {
	let pool = RelayerPool::load()?;
	let RelayerPool { btc_relayer, other_relayers } = &pool;

	thread::scope::<_, Result<()>>(|s| {
		let mut btc_rt = Some(s.spawn(move || btc_relayer.start()));
		let mut others_rt =
			other_relayers.iter().map(|r| Some(s.spawn(move || r.start()))).collect::<Vec<_>>();

		loop {
			btc_rt = supervise(btc_relayer, btc_rt.take(), |r| s.spawn(|| r.start()))?;

			for (i, t) in others_rt.iter_mut().enumerate() {
				*t = supervise(&*other_relayers[i], t.take(), |r| s.spawn(|| r.start()))?;
			}

			thread::sleep(Duration::from_millis(200));
		}
	})?;

	Ok(())
}

fn supervise<'a, 'b, F>(
	relayer: &'a dyn Relayer,
	thread: Option<ScopedJoinHandle<'b, Result<()>>>,
	restart_fn: F,
) -> Result<Option<ScopedJoinHandle<'b, Result<()>>>>
where
	'a: 'b,
	F: FnOnce(&'a dyn Relayer) -> ScopedJoinHandle<'b, Result<()>>,
{
	if let Some(t) = thread {
		if t.is_finished() {
			if let Err(e) = t.join().map_err(Error::Any)? {
				tracing::error!("an error occurred while running {}: {e:?}", relayer.name());

				return Ok(Some(restart_fn(relayer)));
			}
		}
	}

	Ok(None)
}
