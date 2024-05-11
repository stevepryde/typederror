#[macro_export]
macro_rules! terror {
    ($msg:literal $(,)?) => {
        $crate::TError::from_anyhow($crate::anyhow::anyhow!($msg))
    };
    ($err:expr $(,)?) => {
        $crate::TError::from_anyhow($crate::anyhow::anyhow!($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::TError::from_anyhow($crate::anyhow::anyhow!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::terror!($msg))
    };
    ($err:expr $(,)?) => {
        return Err($crate::terror!($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::terror!($fmt, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::{bail, terror};

    #[derive(Debug, thiserror::Error)]
    enum MyError {
        #[error("something went wrong")]
        One,
        #[error("something else")]
        Two,
    }

    fn do_bail() -> crate::Result<(), String> {
        bail!("fake error");
    }

    fn do_bail2() -> crate::Result<(), MyError> {
        bail!(MyError::Two);
    }

    fn do_terror() -> crate::Result<(), String> {
        Err(terror!("fake error"))
    }

    #[test]
    fn test_bail_macro() {
        let a = do_bail();
        assert!(a.is_err());
        assert_eq!(a.unwrap_err().to_string(), "fake error");

        let e = do_bail2().unwrap_err();
        assert_matches!(e.get_ref(), Some(&MyError::Two));
    }

    #[test]
    fn test_terror_macro() {
        let a = do_terror();
        assert!(a.is_err());
        assert_eq!(a.unwrap_err().to_string(), "fake error");

        let e: crate::TError<MyError> = terror!(MyError::One);
        assert_matches!(e.get_ref(), Some(&MyError::One));
    }
}
