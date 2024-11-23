use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies, Key};

use super::get_secret_key;
use crate::{
    ctx::Ctx,
    error::{AppError, AppResult},
    model::ModelController,
    web::AUTH_TOKEN,
};

pub(crate) async fn mw_require_auth(
    ctx: AppResult<Ctx>,
    req: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    tracing::info!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub(crate) async fn mw_ctx_resolver(
    _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    tracing::info!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let key = Key::from(get_secret_key().as_bytes());
    let private_cookies = cookies.private(&key);

    let auth_token = private_cookies
        .get(AUTH_TOKEN)
        .map(|cookie| cookie.value().to_owned());

    // Compute Result<Ctx>.
    let result_ctx = match auth_token
        .ok_or(AppError::AuthFailNoAuthTokenCookie)
        .and_then(|token| parse_token(&token))
    {
        Ok((user_id, exp)) => timestamp_is_valid(&exp).map(|()| {
            let timestamp = Utc::now().timestamp();
            let token = format!("user-{user_id}.{timestamp}");
            set_private_cookie(&cookies, token);
            Ctx::new(user_id)
        }),
        Err(error) => Err(error),
    };

    // Remove the cookie if something went wrong other than NoAuthTokenCookie.
    if result_ctx.is_err() && !matches!(result_ctx, Err(AppError::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    // Store the ctx_result in the request extension.
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> AppResult<Self> {
        tracing::info!("->> {:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<AppResult<Self>>()
            .ok_or(AppError::AuthFailCtxNotInRequestExt)?
            .clone()
    }
}

fn parse_token(token: &str) -> AppResult<(usize, String)> {
    let (_whole, user_id, exp) = regex_captures!(
        r#"^user-(\d+)\.(.+)"#, // a literal regex
        &token
    )
    .ok_or(AppError::AuthFailTokenWrongFormat)?;

    let user_id = user_id
        .parse::<usize>()
        .map_err(|_error| AppError::AuthFailTokenWrongFormat)?;

    Ok((user_id, exp.to_owned()))
}

pub(crate) fn set_private_cookie(cookies: &Cookies, token: String) {
    let key = &Key::from(get_secret_key().as_bytes());

    let private_cookies = cookies.private(key);

    let mut cookie = Cookie::new(AUTH_TOKEN, token);
    cookie.set_path("/");

    private_cookies.add(cookie);
}

fn timestamp_is_valid(exp: &str) -> AppResult<()> {
    // Parse the timestamp string as an integer
    let Ok(timestamp) = exp.parse::<i64>() else {
        return Err(AppError::AuthFailInvalidTimestamp);
    };
    let current_timestamp = Utc::now().timestamp();

    let difference = current_timestamp - timestamp;

    // Check if the difference is greater than 1 hour (3600 seconds)
    if difference < 3600 {
        Ok(())
    } else {
        Err(AppError::AuthFailExpiredTokenCookie)
    }
}
