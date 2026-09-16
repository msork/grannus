use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug)]
pub(crate) struct SignalShutdown {
    requested: Arc<AtomicBool>,
}

impl SignalShutdown {
    #[cfg(unix)]
    pub(crate) fn install() -> Result<Self, Box<dyn std::error::Error>> {
        let requested = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&requested))?;
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&requested))?;
        Ok(Self { requested })
    }

    #[cfg(not(unix))]
    pub(crate) fn install() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::disabled())
    }

    pub(crate) fn requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
    }
}
