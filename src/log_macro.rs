#[macro_export(crate)]
macro_rules! info {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::log::info!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = format!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! error {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::log::error!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = format!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! warn {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::log::warn!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = format!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! trace {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::log::trace!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = format!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! debug {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::log::debug!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = format!($($t)*);
    };
}