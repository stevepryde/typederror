//! A wrapper around `anyhow` but with a "primary" error type.
//!
//! ## Motivation
//!
//! This library aims to be the glue between `anyhow` and `thiserror`.
//! It allows you to define a primary error type for variants that the caller
//! should match on, while still capturing any other errors that may have
//! occurred along the way.
//!
//! ### Documenting the error type of a function
//!
//! If you simply return an `anyhow::Error`, the caller has no idea what
//! kind of error to expect. They would need to read your code to determine
//! what the possible error types are.
//!
//! By using `TError`, you can specify the primary error type that the caller
//! should match on. This has the effect of documenting the primary error type
//! for your function.
//!
//! ```ignore
//! fn my_fallible_function() -> typederror::Result<(), MyError> {
//!     // Do something that might fail.
//!     let s = std::fs::read_to_string("file.txt").map_err(|e| MyError::IoError(e))?;
//!     // NOTE: if `MyError` implements `From<std::io::Error>`,
//!     // you can do `std::fs::read_to_string("file.txt").terror()?` instead.
//!     some_operation(s)?; // An error we don't need to match on.
//!     Ok(())
//! }
//! ```
//!
//! The primary error type could be an enum that derives
//! `thiserror::Error`, where only the meaningful errors are captured by
//! the enum and any other errors are captured by the `anyhow::Error`
//! underneath.
//!
//! You can also implement `DefaultError` so that all other errors are
//! captured in a special "catch-all" variant of the primary error type.
//!
//! ```ignore
//! #[derive(Debug, thiserror::Error)]
//! enum MyError {
//!    #[error("IO error: {0}")]
//!    IoError(#[from] std::io::Error),
//!    #[error("{0}")]
//!    Misc(typederror::anyhow::Error)
//! }
//!
//! impl DefaultError for MyError {
//!     fn from_anyhow(err: typederror::anyhow::Error) -> Self {
//!         Self::Misc(err)
//!     }
//! }
//! ```
//!
//! ### Downcasting to the primary error type
//!
//! Since `TError` already knows the primary error type, it can provide
//! convenience methods for downcasting to that type. This allows you to
//! more easily work with errors of a single type without needing to match
//! on several different error types.
//!
//! ```ignore
//! if let Err(err) = my_fallible_function() { // returns Result<T, TError<MyError>>
//!     match err.get() {
//!         MyError::IoError(e) => { // e is of type `std::io::Error`
//!             // Handle the error.
//!         }
//!         MyError::Misc(e) => { // e is of type `anyhow::Error`
//!             // Handle the error.
//!         }
//!     }
//! }
//! ```
//!
//! You can also downcast to other types if needed, the same as you
//! would with `anyhow`.
//!
//! ```ignore
//! match err.downcast_ref::<serde::Error>() {
//!     Ok(e) => {
//!         // Handle serde error.
//!     }
//!     Err(e) => {
//!         // Handle other error.
//!     }
//! }
//! ```
//!
//! ### Start simple and add error variants later
//!
//! To get you started, you can use `TError<()>` as the primary error type.
//! Or use `typederror::Result<T>` as the return type of your function.
//! This will effectively work the same as `anyhow`, allowing you to
//! write your code and worry about error types later.
//!
//! ```ignore
//! fn do_something() -> typederror::Result<()> {
//!     // Do something.
//!     my_fallible_function()?;
//!     Ok(())
//! }
//! ```
//!
//! Later, when you want to create specific variants for your function
//! for easier matching by the caller, you can create an enum,
//! derive `thiserror::Error`, and use that as the primary error type instead.
//! You will need to add any necessary conversions, but you only need to add
//! the variants you want to match on.
//!
//! All other errors will still be captured as per `anyhow` behaviour, or
//! they can be captured in a special "catch-all" variant of your enum by
//! implementing the `DefaultError` trait on the enum.
//!
//! ## Caveats
//!
//! Unfortunately the `?` operator cannot automatically convert error types
//! to your primary error type.
//!
//! For example:
//! ```ignore
//! #[derive(Debug, thiserror::Error)]
//! enum MyError {
//!     #[error("IO error: {0}")]
//!     IoError(#[from] std::io::Error),
//!     #[error("{0}")]
//!     Misc(anyhow::Error)
//! }
//!
//! impl DefaultError for MyError {
//!     fn from_anyhow(err: anyhow::Error) -> Self {
//!         Self::Misc(err)
//!     }
//! }
//!
//! fn my_fallible_function() -> typederror::Result<(), MyError> {
//!    let s = std::fs::read_to_string("file.txt")?;
//!    // Do something else with s.
//!    Ok(())
//! }
//!
//! fn main() {
//!     if let Err(e) = my_fallible_function() {
//!         match e.get() {
//!             // ...
//!         }
//!     }
//! }
//! ```
//!
//! In the above example, the `?` operator will not automatically convert the
//! `std::io::Error` to `MyError::IoError`, as it would if you had used
//! `MyError` as the error type directly. The error would instead match as
//! `MyError::Misc` in the call to `e.get()`.
//!
//! To capture the `IoError` correctly, change the first line of the function to
//! ```ignore
//! let s = std::fs::read_to_string("file.txt").terror()?;
//! ```
//!
mod error;
pub use error::*;
pub mod macros;

pub mod prelude {
    pub use crate::error::{Context, DefaultError, IntoTError, TError, WrapTError};
    pub use crate::terror;
    pub use crate::Result as TEResult;
}

/// Re-export of anyhow macros.
pub mod anyhow {
    pub use anyhow::{anyhow, bail};
}
