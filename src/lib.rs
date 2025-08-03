pub mod extractors;
// mod flatdata;

pub use baad_core;
pub use baad_core::Result;

pub fn init_default_logging() -> Result<()> {
    baad_core::config::init_logging_default().map_err(baad_core::LoggingError::from)
}
