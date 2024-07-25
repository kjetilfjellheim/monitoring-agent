/**
 * Application error struct.
 */
#[derive(Debug)]
pub struct CommonLibError {
    pub message: String,
}

impl CommonLibError {
    pub fn new(message: &str) -> CommonLibError {
        CommonLibError {
            message: message.to_string(),
        }
    }
}