/**
 * Application error struct.
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CommonLibError {
    pub message: String,
}

impl CommonLibError {
    /**
     * Create a new `CommonLibError`.
     *
     * `message`: The error message.
     */
    #[must_use] pub fn new(message: &str) -> CommonLibError {
        CommonLibError {
            message: message.to_string(),
        }
    }
}