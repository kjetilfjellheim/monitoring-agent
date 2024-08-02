/**
 * Application error struct.
 */
#[derive(Debug)]
pub struct ApplicationError {
    /// The error message.
    pub message: String,
}

impl ApplicationError {
    /**
     * Create a new application error.
     * 
     * `message`: The error message.
     * 
     * result: The result of creating the application error.
     */
    pub fn new(message: &str) -> ApplicationError {
        ApplicationError {
            message: message.to_string(),
        }
    }

    pub fn get_message(&self) -> String {
        self.message.clone()
    }
}
