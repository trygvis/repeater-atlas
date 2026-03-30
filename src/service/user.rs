use crate::auth;
use crate::dao;
use crate::dao::user::{NewUser, User};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use diesel::QueryResult;
use diesel::result::{DatabaseErrorKind, Error};
use diesel_async::AsyncPgConnection;
use regex::Regex;
use std::sync::LazyLock;
use tracing::info;

#[derive(Debug)]
pub enum CreateUserResult {
    Ok(User),
    InvalidCallSign,
    InvalidEmail,
    InvalidPassword,
    DuplicateUser,
}

pub async fn create_user(
    c: &mut AsyncPgConnection,
    call_sign: &str,
    email: &str,
    password: &str,
) -> QueryResult<CreateUserResult> {
    let call_sign = auth::normalize_call_sign(call_sign);
    let email = email.trim().to_lowercase();

    if !is_valid_call_sign(&call_sign) {
        return Ok(CreateUserResult::InvalidCallSign);
    }

    if !is_valid_email(&email) {
        return Ok(CreateUserResult::InvalidEmail);
    }

    if !is_valid_password(password) {
        return Ok(CreateUserResult::InvalidPassword);
    }

    info!("Creating user: {call_sign}");

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| {
            Error::SerializationError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            )))
        })?
        .to_string();

    match dao::user::insert(
        c,
        NewUser {
            call_sign,
            email,
            password_hash: hash,
        },
    )
    .await
    {
        Ok(user) => Ok(CreateUserResult::Ok(user)),
        Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Ok(CreateUserResult::DuplicateUser)
        }
        Err(error) => Err(error),
    }
}

fn is_valid_call_sign(call_sign: &str) -> bool {
    static CALL_SIGN_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[A-Z0-9]+(?:/[A-Z0-9]+)*$").expect("valid regex"));

    (3..=10).contains(&call_sign.len())
        && call_sign.chars().any(|c| c.is_ascii_alphabetic())
        && call_sign.chars().any(|c| c.is_ascii_digit())
        && CALL_SIGN_RE.is_match(call_sign)
}

fn is_valid_email(email: &str) -> bool {
    static EMAIL_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("valid regex"));

    EMAIL_RE.is_match(email)
}

fn is_valid_password(password: &str) -> bool {
    password.len() >= 8
}
