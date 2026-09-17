use crate::pages::certificate;
use rullst::db::{FromRow, Orm, sqlx};
use rullst::server::{Extension, IntoResponse, Path, Response, StatusCode};

#[derive(Debug, FromRow)]
struct CertificateView {
    public_id: String,
    badge_kind: String,
    environment: String,
    certificate_status: String,
    entitlement_status: String,
    created_at: String,
    holder_name: String,
}

pub fn valid_public_certificate_id(value: &str) -> bool {
    value.len() == 40
        && value.starts_with("RST-SBX-")
        && value[8..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_lowercase())
}

pub async fn account_certificate_public_id(
    user_id: i32,
) -> Result<Option<String>, rullst_orm::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT tc.public_id FROM tester_certificates tc INNER JOIN entitlements e ON e.id = tc.entitlement_id WHERE e.user_id = $1 AND e.product_sku = 'gateway-report-stripe' AND e.status = 'active' AND tc.status = 'active' LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(Orm::pool()?)
    .await
    .map_err(Into::into)
}

async fn certificate_for_owner(user_id: i32) -> Result<Option<CertificateView>, rullst_orm::Error> {
    sqlx::query_as::<_, CertificateView>(
        "SELECT tc.public_id, tc.badge_kind, tc.environment, tc.status AS certificate_status, e.status AS entitlement_status, tc.created_at::text AS created_at, u.name AS holder_name FROM tester_certificates tc INNER JOIN entitlements e ON e.id = tc.entitlement_id INNER JOIN users u ON u.id = e.user_id WHERE e.user_id = $1 AND e.product_sku = 'gateway-report-stripe' LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(Orm::pool()?)
    .await
    .map_err(Into::into)
}

async fn certificate_for_public_id(
    public_id: &str,
) -> Result<Option<CertificateView>, rullst_orm::Error> {
    sqlx::query_as::<_, CertificateView>(
        "SELECT tc.public_id, tc.badge_kind, tc.environment, tc.status AS certificate_status, e.status AS entitlement_status, tc.created_at::text AS created_at, '' AS holder_name FROM tester_certificates tc INNER JOIN entitlements e ON e.id = tc.entitlement_id WHERE tc.public_id = $1 LIMIT 1",
    )
    .bind(public_id)
    .fetch_optional(Orm::pool()?)
    .await
    .map_err(Into::into)
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        rullst::server::header::CACHE_CONTROL,
        rullst::server::HeaderValue::from_static("no-store"),
    );
    response
}

pub async fn owner_certificate(
    Extension(user_id): Extension<i32>,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> Response {
    let nonce = csp_nonce
        .as_ref()
        .map(|Extension(nonce)| nonce.as_str())
        .unwrap_or_default();
    match certificate_for_owner(user_id).await {
        Ok(Some(value)) => no_store(
            certificate::owner_certificate_page(
                &value.holder_name,
                &value.public_id,
                &value.badge_kind,
                &value.environment,
                &value.certificate_status,
                &value.entitlement_status,
                &value.created_at,
                nonce,
            )
            .into_response(),
        ),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            eprintln!("Certificate owner lookup failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

pub async fn verify_certificate(
    Path(public_id): Path<String>,
    csp_nonce: Option<Extension<rullst::security::CspNonce>>,
) -> Response {
    if !valid_public_certificate_id(&public_id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match certificate_for_public_id(&public_id).await {
        Ok(Some(value)) => {
            let active =
                value.certificate_status == "active" && value.entitlement_status == "active";
            let status = if active {
                StatusCode::OK
            } else {
                StatusCode::GONE
            };
            let mut response = certificate::public_verification_page(
                &value.public_id,
                &value.badge_kind,
                &value.environment,
                active,
                &value.created_at,
                csp_nonce
                    .as_ref()
                    .map(|Extension(nonce)| nonce.as_str())
                    .unwrap_or_default(),
            )
            .into_response();
            *response.status_mut() = status;
            no_store(response)
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            eprintln!("Public certificate lookup failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::valid_public_certificate_id;

    #[test]
    fn public_certificate_ids_are_strict_and_non_enumerable() {
        assert!(valid_public_certificate_id(
            "RST-SBX-0123456789ABCDEF0123456789ABCDEF"
        ));
        assert!(!valid_public_certificate_id(
            "RST-SBX-0123456789abcdef0123456789abcdef"
        ));
        assert!(!valid_public_certificate_id("RST-SBX-1234"));
        assert!(!valid_public_certificate_id(
            "RST-LIVE-0123456789ABCDEF0123456789ABCDEF"
        ));
    }
}
