// #[macro_export(crate)]
macro_rules! info {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::tracing::info!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = stringify!($($t)*);
    };
}
// #[macro_export(crate)]
macro_rules! debug {
    ($($t:tt)*) => {
        #[cfg(feature = "logger")]
        ::tracing::debug!($($t)*);
        #[cfg(not(feature = "logger"))]
        let _ = stringify!($($t)*);
    };
}
