use sha2::{Digest, Sha256};

const SESSION_COOKIE_NAME: &str = "rullst_session";
const SESSION_HASH_DOMAIN: &[u8] = b"rullst.saas.session-registry.v1\0";

pub fn hash_session_token(token: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(SESSION_HASH_DOMAIN);
    digest.update(token.as_bytes());
    hex::encode(digest.finalize())
}

pub fn token_from_set_cookie(cookie: &str) -> Option<&str> {
    let first_part = cookie.split(';').next()?;
    let (name, value) = first_part.split_once('=')?;
    if name.trim() != SESSION_COOKIE_NAME
        || value.is_empty()
        || value.len() > 4096
        || !value.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return None;
    }
    Some(value)
}

pub async fn register_cookie(user_id: i32, cookie: &str) -> Result<(), String> {
    let token = token_from_set_cookie(cookie)
        .ok_or_else(|| "the generated session cookie has an invalid shape".to_owned())?;
    let pool = rullst::db::Orm::pool().map_err(|error| error.to_string())?;
    rullst::db::sqlx::query("DELETE FROM auth_sessions WHERE expires_at <= CURRENT_TIMESTAMP")
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    rullst::db::sqlx::query(
        "INSERT INTO auth_sessions (session_hash, user_id, expires_at) VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '30 days')",
    )
    .bind(hash_session_token(token))
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn is_active(user_id: i32, token: &str) -> Result<bool, String> {
    let pool = rullst::db::Orm::pool().map_err(|error| error.to_string())?;
    rullst::db::sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM auth_sessions WHERE session_hash = $1 AND user_id = $2 AND expires_at > CURRENT_TIMESTAMP)",
    )
    .bind(hash_session_token(token))
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|error| error.to_string())
}

pub async fn revoke(token: &str) -> Result<(), String> {
    let pool = rullst::db::Orm::pool().map_err(|error| error.to_string())?;
    rullst::db::sqlx::query("DELETE FROM auth_sessions WHERE session_hash = $1")
        .bind(hash_session_token(token))
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_cookie_parser_accepts_only_the_session_cookie() {
        assert_eq!(
            token_from_set_cookie("rullst_session=v1.abc; Path=/; HttpOnly"),
            Some("v1.abc")
        );
        assert_eq!(token_from_set_cookie("other=v1.abc; Path=/"), None);
        assert_eq!(token_from_set_cookie("rullst_session=; Path=/"), None);
    }

    #[test]
    fn session_hash_is_domain_separated_and_stable() {
        let first = hash_session_token("v1.example");
        assert_eq!(first.len(), 64);
        assert_eq!(first, hash_session_token("v1.example"));
        assert_ne!(first, hash_session_token("v1.other"));
    }
}
