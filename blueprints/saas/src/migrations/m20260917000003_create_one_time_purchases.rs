use rullst::db::async_trait;
use rullst::db::schema::{Migration, Schema};
use rullst::db::{Orm, sqlx};

pub struct MigrationImpl;

#[async_trait]
impl Migration for MigrationImpl {
    fn name(&self) -> &'static str {
        "m20260917000003_create_one_time_purchases"
    }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::create("purchase_attempts", |table| {
            table.id();
            table.integer("user_id").not_null();
            table.string("provider").not_null();
            table.string("product_sku").not_null();
            table.string("provider_price_id").not_null();
            table.integer("expected_amount_minor").not_null();
            table.string("expected_currency").not_null();
            table.string("checkout_token_hash").not_null();
            table.string("provider_session_id").nullable();
            table.string("provider_payment_id").nullable();
            table.string("status").not_null();
            table.timestamps();
        })
        .await?;
        Schema::create("provider_events", |table| {
            table.id();
            table.string("provider").not_null();
            table.string("event_id").not_null();
            table.string("event_type").not_null();
            table.string("payload_sha256").not_null();
            table.timestamps();
        })
        .await?;
        Schema::create("entitlements", |table| {
            table.id();
            table.integer("user_id").not_null();
            table.string("product_sku").not_null();
            table.string("provider").not_null();
            table.string("provider_payment_id").not_null();
            table.string("artifact_version").not_null();
            table.string("status").not_null();
            table.timestamps();
        })
        .await?;

        let pool = Orm::pool()?;
        sqlx::query(
            "CREATE UNIQUE INDEX purchase_attempts_provider_session_unique ON purchase_attempts(provider, provider_session_id) WHERE provider_session_id IS NOT NULL",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX purchase_attempts_open_offer_unique ON purchase_attempts(user_id, product_sku) WHERE status IN ('pending', 'unknown', 'checkout_created')",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX provider_events_provider_event_unique ON provider_events(provider, event_id)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX entitlements_user_product_unique ON entitlements(user_id, product_sku)",
        )
        .execute(pool)
        .await?;
        sqlx::query(
            "CREATE UNIQUE INDEX entitlements_provider_payment_unique ON entitlements(provider, provider_payment_id)",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        Schema::drop_if_exists("entitlements").await?;
        Schema::drop_if_exists("provider_events").await?;
        Schema::drop_if_exists("purchase_attempts").await
    }
}
