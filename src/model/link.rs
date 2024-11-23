use super::ModelController;

impl ModelController {
    pub(crate) fn get_links(&self, username: &str, user_id: usize) -> String {
        format!("Links for user {username} with id: {user_id}")
    }
}
