//! Cross-platform instant type.
//!
//! On native platforms this re-exports `std::time::Instant` unchanged.
//! On WASM, `std::time::Instant` panics at construction because the platform
//! does not implement the underlying syscall. We replace it with a thin wrapper
//! around `js_sys::Date::now()` (milliseconds since Unix epoch, f64).

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub use wasm::Instant;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::time::Duration;

    #[derive(Clone, Copy, Debug)]
    pub struct Instant(f64);

    impl Instant {
        pub fn now() -> Self {
            Instant(js_sys::Date::now())
        }

        pub fn elapsed(&self) -> Duration {
            let millis = (js_sys::Date::now() - self.0).max(0.0) as u64;
            Duration::from_millis(millis)
        }
    }
}
