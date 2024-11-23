use std::{env, sync::OnceLock};

pub(crate) mod mw_auth;
pub(crate) mod routes_link;
pub(crate) mod routes_user;

pub(crate) const AUTH_TOKEN: &str = "auth-token";

static MY_KEY: OnceLock<String> = OnceLock::new();

pub(crate) fn get_secret_key() -> &'static str {
    MY_KEY.get_or_init(|| {
        env::var("SECRET_KEY").expect("No SECRET_KEY for cookie encryption provided.")
    })
}
