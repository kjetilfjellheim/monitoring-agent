/**
 * Application error struct.
 */
#[derive(Debug)]
pub struct ApplicationError {
    pub message: String,
}

impl ApplicationError {
    pub fn new<'a>(message: &str) -> ApplicationError {
        ApplicationError {
            message: message.to_string(),
        }
    }
}
