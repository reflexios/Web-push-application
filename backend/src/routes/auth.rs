use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::{StatusCode, request::Parts};
use axum::{Json, async_trait};
use axum_extra::extract::cookie::{Cookie, Key, PrivateCookieJar, SameSite};

use crate::AppState;
use crate::entities::admin::LoginRequest;

const SESSION_COOKIE: &str = "session";
const SESSION_VALUE: &str = "authenticated";

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.config.cookie_key.clone()
    }
}

pub struct RequireAuth;

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
    Key: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        match jar.get(SESSION_COOKIE) {
            Some(cookie) if cookie.value() == SESSION_VALUE => Ok(RequireAuth),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(PrivateCookieJar, StatusCode), StatusCode> {
    if !state.config.is_admin(payload) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let cookie = Cookie::build((SESSION_COOKIE, SESSION_VALUE))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.config.secure_mode)
        .build();

    let jar = jar.add(cookie);
    Ok((jar, StatusCode::OK))
}

pub async fn logout(jar: PrivateCookieJar) -> (PrivateCookieJar, StatusCode) {
    let jar = jar.remove(Cookie::from(SESSION_COOKIE));
    (jar, StatusCode::OK)
}
