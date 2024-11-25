use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::{
    database::DatabaseClient,
    error::{AppError, AppResult},
    web::routes_auth::LoginData,
};

use super::ModelController;

impl ModelController {
    pub(crate) async fn login(&self, login_data: LoginData) -> AppResult<usize> {
        let database_client = DatabaseClient::new(self.pool.clone());

        let (user_id, password_hash) = database_client.get_auth_data(login_data.email).await?;

        if user_id < 0 {
            return Err(AppError::AuthFailInvalidId);
        }

        let argon2 = Argon2::default();

        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|_error| AppError::AuthFailParsingPasswordHashFail)?;

        argon2
            .verify_password(login_data.password.as_bytes(), &parsed_hash)
            .map_err(|_error| AppError::AuthFailVerifyPasswordFail)?;

        // Okay to cast since we validated id >= 0
        user_id
            .try_into()
            .map_err(|_error| AppError::AuthFailInvalidId)
    }
}
