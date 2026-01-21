use crate::auth;
use crate::dao;
use crate::dao::user::{NewUser, User};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use diesel::QueryResult;
use diesel::result::Error;
use diesel_async::AsyncPgConnection;
use tracing::info;

pub async fn create_user(
    c: &mut AsyncPgConnection,
    call_sign: &str,
    email: &str,
    password: &str,
) -> QueryResult<User> {
    let call_sign = auth::normalize_call_sign(call_sign);

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

    dao::user::insert(
        c,
        NewUser {
            call_sign,
            email: email.to_string(),
            password_hash: hash,
        },
    )
    .await
}
