macro_rules! build_custom_error {
    ($err_name:ident, $err_msg:literal) => {
        #[derive(Debug)]
        struct $err_name;
        impl Error for $err_name {}
        impl fmt::Display for $err_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $err_msg)
            }
        }
    };
}

pub(crate) use build_custom_error;
