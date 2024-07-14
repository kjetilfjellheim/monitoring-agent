#[derive(Debug)]
pub struct ApplicationError {
    pub errorcode: i32,
    pub message: String
}

impl ApplicationError {
    pub fn new<'a>(errorcode: i32, message: &str) -> ApplicationError {
        ApplicationError {
            errorcode,
            message: message.to_string()
        }
    }
}