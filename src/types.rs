pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub fn error_from(msg: &str) -> Error {
    Box::<dyn std::error::Error>::from(msg.to_owned())
}

pub fn assert_or_err(assertion: bool, err_msg: &str) -> Result<()> {
    if assertion {
        Ok(())
    } else {
        Err(error_from(err_msg))
    }
}