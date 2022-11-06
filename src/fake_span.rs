#[cfg(not(feature = "logger"))]
pub struct FakeSpan;

#[cfg(not(feature = "logger"))]
impl FakeSpan {
    pub fn in_scope<F: FnOnce() -> T, T>(&self, func: F) -> T {
        func()
    }
}
