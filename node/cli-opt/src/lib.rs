use clap::ValueEnum;
use std::fmt;

/// Block authoring sealing scheme to be used by the dev service.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SealingMode {
	/// Author a block immediately upon receiving a transaction into the transaction pool
	Instant,
	/// Author a block upon receiving an RPC command
	Manual,
	/// Author blocks at a regular interval specified in seconds
	Interval,
}

impl fmt::Display for SealingMode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SealingMode::Instant => write!(f, "Instant"),
			SealingMode::Manual => write!(f, "Manual"),
			SealingMode::Interval => write!(f, "Interval"),
		}
	}
}
