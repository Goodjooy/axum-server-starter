#[macro_export(crate)]
macro_rules! info {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::tracing::info!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = stringify!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! error {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::tracing::error!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = stringify!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! warn {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::tracing::warn!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = stringify!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! trace {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::tracing::trace!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = stringify!($($t)*);
    };
}
#[macro_export(crate)]
macro_rules! debug {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::tracing::debug!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = stringify!($($t)*);
    };
}
