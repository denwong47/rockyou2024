//! Centralised logging for the parser.
//!

use std::sync::OnceLock;

/// Lock for initialising the logger.
static INIT: OnceLock<()> = OnceLock::new();

/// Initialises the logger.
pub fn init() {
    INIT.get_or_init(env_logger::init);
}

#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::logger::init();
        log::trace!(target: $target, $($arg)+);
    );

    ($($arg:tt)+) => (
        $crate::logger::init();
        log::trace!($($arg)+);
    )
}

#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::logger::init();
        log::debug!(target: $target, $($arg)+);
    );

    ($($arg:tt)+) => (
        $crate::logger::init();
        log::debug!($($arg)+);
    )
}

#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::logger::init();
        log::info!(target: $target, $($arg)+);
    );

    ($($arg:tt)+) => (
        $crate::logger::init();
        log::info!($($arg)+);
    )
}

#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::logger::init();
        log::warn!(target: $target, $($arg)+);
    );

    ($($arg:tt)+) => (
        $crate::logger::init();
        log::warn!($($arg)+);
    )
}

#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::logger::init();
        log::error!(target: $target, $($arg)+);
    );

    ($($arg:tt)+) => (
        $crate::logger::init();
        log::error!($($arg)+);
    )
}
