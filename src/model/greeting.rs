use super::ModelController;

impl ModelController {
    pub(crate) fn get_greeting(&self, username: &str) -> String {
        format!("Hello, {username}")
    }
}
