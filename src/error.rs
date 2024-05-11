use std::fmt::Display;
use std::{error::Error, fmt::Debug};

/// Convenience type alias for `Result<T, TError<E>>`.
pub type Result<T, E = ()> = std::result::Result<T, TError<E>>;

/// A wrapper around `anyhow::Error` that allows for downcasting to a specific error type.
///
/// The primary use-case is to mix anyhow with a dedicated error type,
/// such as an enum that derives `thiserror::Error`. The generic type
/// parameter acts as documentation for the returned error type for
/// the caller to match on, while the underlying anyhow::Error also
/// allows for other errors to be captured along with any context.
pub struct TError<E = ()> {
    phantom: std::marker::PhantomData<E>,
    error: anyhow::Error,
}

impl<E> Debug for TError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl<E> Display for TError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl<T> From<TError<T>> for anyhow::Error {
    fn from(err: TError<T>) -> Self {
        err.error
    }
}

impl<E: Debug + Display + Send + Sync + 'static> TError<E> {
    pub fn from_anyhow(error: anyhow::Error) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            error,
        }
    }

    pub fn from_msg(msg: &str) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            error: anyhow::anyhow!("{msg}"),
        }
    }

    /// Get the most recent error of the default type E.
    pub fn try_get(self) -> Result<E, TError<E>> {
        self.error.downcast().map_err(|e| TError {
            phantom: std::marker::PhantomData,
            error: e,
        })
    }

    /// Get the most recent error of the default type E.
    pub fn get_ref(&self) -> Option<&E> {
        self.error.downcast_ref::<E>()
    }

    /// Get the most recent error of type T.
    pub fn downcast_ref<T: Debug + Display + Send + Sync + 'static>(&self) -> Option<&T> {
        self.error.downcast_ref::<T>()
    }

    pub fn downcast<T: Debug + Display + Send + Sync + 'static>(self) -> Result<T, Self> {
        self.error.downcast::<T>().map_err(|e| TError {
            phantom: std::marker::PhantomData,
            error: e,
        })
    }

    /// Add context to the error.
    pub fn context<C>(self, context: C) -> TError<E>
    where
        C: Display + Send + Sync + 'static,
    {
        let error = self.error.context(context);
        TError {
            phantom: std::marker::PhantomData,
            error,
        }
    }

    /// Add context to the error.
    pub fn with_context<F, R>(self, context: F) -> TError<E>
    where
        F: FnOnce() -> R,
        R: Display + Send + Sync + 'static,
    {
        self.context(context())
    }

    /// Change the generic error type.
    pub fn change_err<T>(self) -> TError<T> {
        TError::<T> {
            phantom: std::marker::PhantomData,
            error: self.error,
        }
    }
}

impl<E: Default + Debug + Display + Send + Sync + 'static> TError<E> {}

impl<SRC: Error + Send + Sync + 'static, DST: Error + 'static> From<SRC> for TError<DST> {
    fn from(err: SRC) -> Self {
        let error = anyhow::Error::new(err);
        Self {
            phantom: std::marker::PhantomData,
            error,
        }
    }
}

pub(crate) mod private {
    pub trait Sealed {}
}

/// Extension trait for `Result` to add context to the `Result`.
pub trait Context<T, E, X: Display>: private::Sealed {
    /// Wrap the error value with additional context.
    fn context<C>(self, context: C) -> std::result::Result<T, TError<X>>
    where
        C: Display + Send + Sync + 'static;

    /// Wrap the error value with additional context that is evaluated lazily
    /// only once an error does occur.
    fn with_context<C, F>(self, f: F) -> std::result::Result<T, TError<X>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> private::Sealed for std::result::Result<T, E> {}

impl<T, E: Error + Send + Sync + 'static, X: Error> Context<T, E, X> for std::result::Result<T, E> {
    fn context<C>(self, context: C) -> std::result::Result<T, TError<X>>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            let error = anyhow::Error::new(err);
            let error = error.context(context.to_string());
            TError {
                phantom: std::marker::PhantomData,
                error,
            }
        })
    }

    fn with_context<C, F>(self, f: F) -> std::result::Result<T, TError<X>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.context(f())
    }
}

/// Extension trait to allow capturing errors into a "default" bucket.
///
/// This is useful for collecting errors that you want to report on but
/// don't need to match on. You can use this with a catch-all enum variant
/// that contains either contains `anyhow::Error` or
/// `Box<dyn Error + Send + Sync + 'static>`.
///
/// Any errors that don't match the primary error type will be captured
/// in the default error variant.
pub trait DefaultError {
    /// Construct an error variant from the given error.
    fn from_anyhow(err: anyhow::Error) -> Self;
}

impl<E: DefaultError + Debug + Display + Send + Sync + 'static> TError<E> {
    /// Get the most recent error of the default type E, or the default error.
    ///
    /// If no error was found of type E, then the error is converted into
    /// type E using the DefaultError trait instead.
    pub fn get(self) -> E {
        self.try_get()
            .unwrap_or_else(|err| E::from_anyhow(err.error))
    }
}

/// Trait to convert something to a `Result<T, TError<E>>`.
pub trait IntoTError<T, E>: private::Sealed {
    fn terror(self) -> std::result::Result<T, TError<E>>;
}

impl<T, EIn, EOut> IntoTError<T, EOut> for std::result::Result<T, EIn>
where
    EIn: Into<EOut>,
    EOut: std::error::Error + Send + Sync + 'static,
{
    /// Convert `Result<T, EIn>` into `Result<T, TError<EOut>>` where `EIn: Into<EOut>`.
    fn terror(self) -> std::result::Result<T, TError<EOut>> {
        self.map_err(|e| TError {
            phantom: std::marker::PhantomData,
            error: anyhow::Error::new(e.into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[derive(Debug, thiserror::Error)]
    enum MyError {
        #[error("something went wrong")]
        One,
        #[error("Error two")]
        Two(Box<dyn Error + Send + Sync + 'static>),
        #[error("io error: {0}")]
        Three(#[from] std::io::Error),
    }

    impl DefaultError for MyError {
        fn from_anyhow(err: anyhow::Error) -> Self {
            MyError::Two(err.into())
        }
    }

    #[derive(Debug, PartialEq)]
    struct OtherError;

    impl Display for OtherError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "OtherError")
        }
    }

    impl Error for OtherError {}

    fn do_other_task(fail: bool) -> std::result::Result<(), OtherError> {
        if fail {
            Err(OtherError)
        } else {
            Ok(())
        }
    }

    fn fallible_fn(other: bool) -> std::result::Result<(), TError<MyError>> {
        do_other_task(other)?;

        Err(MyError::One).context("failed")
    }

    #[test]
    fn test_err() {
        let err = fallible_fn(false).unwrap_err();
        assert_matches!(err.get_ref(), Some(&MyError::One));
        assert_eq!(format!("{err}"), "failed");

        let e2 = err
            .context("add more context")
            .context("and even more context");

        assert_matches!(e2.get_ref(), Some(&MyError::One));

        let e3 = e2.context(MyError::Two(anyhow::anyhow!("other error").into()));
        assert_matches!(e3.get_ref(), Some(&MyError::Two(_)));

        let err = fallible_fn(true).unwrap_err();
        assert_matches!(err.get_ref(), None);
        assert_eq!(err.downcast_ref(), Some(&OtherError));
        assert_matches!(err.get(), MyError::Two(_)); // We got some other error.
    }

    #[test]
    fn test_terror() {
        let path = std::path::Path::new("/invalid-dir-doesnt-exist");
        // Using the `.terror()` method, we can convert into `MyError` instead of `std::io::Error`.
        let err: TError<MyError> = std::fs::read_to_string(path).terror().unwrap_err();
        assert_matches!(err.get_ref(), Some(&MyError::Three(_)));
    }
}
