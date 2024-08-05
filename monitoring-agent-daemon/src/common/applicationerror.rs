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

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod applicationerror_tests {
        use super::super::*;

        #[test]
        fn test_new() {
            let message = "test message";
            let application_error = ApplicationError::new(message);
            assert_eq!(application_error.get_message(), message);
        }
    }
}