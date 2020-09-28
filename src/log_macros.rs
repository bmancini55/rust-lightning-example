
use std;
use bitcoin::secp256k1::key::PublicKey;

pub(crate) struct DebugPubKey<'a>(pub &'a PublicKey);
impl<'a> std::fmt::Display for DebugPubKey<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		for i in self.0.serialize().iter() {
			write!(f, "{:02x}", i)?;
		}
		Ok(())
	}
}

macro_rules! log_pubkey {
	($obj: expr) => {
		log_macros::DebugPubKey(&$obj)
	}
}


macro_rules! log_internal {
	($logger: expr, $lvl:expr, $($arg:tt)+) => (
		$logger.log(&lightning::util::logger::Record::new($lvl, format_args!($($arg)+), module_path!(), file!(), line!()));
	);
}


macro_rules! log_info {
	($logger: expr, $($arg:tt)*) => (
		log_internal!($logger, lightning::util::logger::Level::Info, $($arg)*);
	)
}
