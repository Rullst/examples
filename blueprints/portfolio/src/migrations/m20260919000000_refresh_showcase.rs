use rullst::db::{async_trait, schema::Migration};

pub struct RefreshShowcase;

#[async_trait]
impl Migration for RefreshShowcase {
    fn name(&self) -> &'static str {
        "m20260919000000_refresh_showcase"
    }

    async fn up(&self) -> Result<(), rullst_orm::error::RullstError> {
        let pool = rullst::db::Orm::pool()?;
        let mut tx = pool.begin().await?;
        // Boot also invokes migrations on an existing SQLite database. Apply this
        // content revision once, preserving subsequent CMS edits and other records.
        rullst::db::sqlx::query(
            "CREATE TABLE IF NOT EXISTS portfolio_content_revisions (name TEXT PRIMARY KEY)",
        )
        .execute(&mut *tx)
        .await?;
        let applied: i64 = rullst::db::sqlx::query_scalar(
            "SELECT COUNT(*) FROM portfolio_content_revisions WHERE name = ?",
        )
        .bind(self.name())
        .fetch_one(&mut *tx)
        .await?;
        if applied != 0 {
            return Ok(());
        }
        rullst::db::sqlx::query("UPDATE profiles SET name = 'Venelouis', title = 'Senior Rust & AI Engineer', subtitle = 'Specializing in hyper-concurrent web backends, Generative AI integration, and high-throughput Rust architectures.', email = 'officialrullst@gmail.com', website = 'https://rullst.win', avatar_url = '/static/rullst.png', github_url = 'https://github.com/Rullst', linkedin_url = 'https://linkedin.com/company/rullst', updated_at = datetime('now') WHERE id = 1")
            .execute(&mut *tx).await?;
        rullst::db::sqlx::query("UPDATE experiences SET role = 'Senior Rust & AI Engineer', company = 'Rullst framework', period = '2026-Present', description = 'Developing the Rullst framework and practical Rust showcases, bringing together concurrent web backends, server-rendered interfaces, and Generative AI integration.', updated_at = datetime('now') WHERE id = 1")
            .execute(&mut *tx).await?;
        // Remove the fictional employment entry shipped with the original template.
        rullst::db::sqlx::query("DELETE FROM experiences WHERE id = 2 AND company = 'Quantum Systems' AND role = 'Full-Stack Developer'")
            .execute(&mut *tx).await?;
        let projects = [
            (
                "Rullst",
                "Explore the Rullst framework and its ecosystem for building Rust web applications.",
                "https://rullst.win/",
                "Rust, Rullst v12, Web framework",
            ),
            (
                "Rullst LMS Showcase",
                "A learning platform demo with a course catalog, lessons, and shared access to Nexus and Studio. Visit Rullst Academy for the full learning experience.",
                "https://lms.rullst.win",
                "Rust, LMS, Rullst v12",
            ),
            (
                "Rullst Portfolio Showcase",
                "This portfolio: a server-rendered Rust application with a database-backed profile, project catalog, Nexus CMS, and optional Career Copilot.",
                "https://portfolio.rullst.win",
                "Rust, HTMX, Portfolio",
            ),
            (
                "Rullst Framework Showcase",
                "Explore interactive framework demos, developer tools, and integrations built with Rullst v12.",
                "https://showcase.rullst.win",
                "Rust, HTMX, Developer tools",
            ),
            (
                "Rullst SaaS Showcase",
                "Explore a SaaS blueprint and its application flows, built with the Rullst framework.",
                "https://saas.rullst.win",
                "Rust, SaaS, Rullst v12",
            ),
        ];
        for (index, (title, description, url, tags)) in projects.iter().enumerate() {
            if index < 2 {
                rullst::db::sqlx::query("UPDATE projects SET title = ?, description = ?, url = ?, tags = ?, updated_at = datetime('now') WHERE id = ?")
                    .bind(title).bind(description).bind(url).bind(tags).bind(index as i64 + 1)
                    .execute(&mut *tx).await?;
            }
            rullst::db::sqlx::query("INSERT INTO projects (title, description, url, tags, is_featured, created_at, updated_at) SELECT ?, ?, ?, ?, 1, datetime('now'), datetime('now') WHERE NOT EXISTS (SELECT 1 FROM projects WHERE url = ?)")
                .bind(title).bind(description).bind(url).bind(tags).bind(url)
                .execute(&mut *tx).await?;
        }
        rullst::db::sqlx::query("INSERT INTO portfolio_content_revisions (name) VALUES (?)")
            .bind(self.name())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn down(&self) -> Result<(), rullst_orm::error::RullstError> {
        // Public content updates cannot restore later CMS edits safely.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn refreshes_existing_content_once_and_preserves_other_records() {
        rullst::db::Orm::init("sqlite::memory:").await.unwrap();
        super::super::m20260701000000_create_portfolio_tables::CreatePortfolioTables
            .up()
            .await
            .unwrap();
        let pool = rullst::db::Orm::pool().unwrap();
        rullst::db::sqlx::query("INSERT INTO projects (id, title, description, url, tags, is_featured) VALUES (42, 'Visitor project', 'Keep me', 'https://example.com', 'Rust', 0)").execute(pool).await.unwrap();
        RefreshShowcase.up().await.unwrap();
        let name: String = rullst::db::sqlx::query_scalar("SELECT name FROM profiles WHERE id = 1")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(name, "Venelouis");
        let urls: Vec<String> = rullst::db::sqlx::query_scalar("SELECT url FROM projects")
            .fetch_all(pool)
            .await
            .unwrap();
        assert_eq!(urls.len(), 6);
        assert!(urls.contains(&"https://example.com".into()));
        for url in [
            "https://rullst.win/",
            "https://lms.rullst.win",
            "https://portfolio.rullst.win",
            "https://showcase.rullst.win",
            "https://saas.rullst.win",
        ] {
            assert!(urls.contains(&url.to_string()));
        }
        rullst::db::sqlx::query("UPDATE profiles SET name = 'CMS edit' WHERE id = 1")
            .execute(pool)
            .await
            .unwrap();
        RefreshShowcase.up().await.unwrap();
        let name: String = rullst::db::sqlx::query_scalar("SELECT name FROM profiles WHERE id = 1")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(name, "CMS edit");
        super::super::m20260701000000_create_portfolio_tables::CreatePortfolioTables
            .up().await.unwrap();
        let experience_count: i64 = rullst::db::sqlx::query_scalar("SELECT COUNT(*) FROM experiences")
            .fetch_one(pool).await.unwrap();
        assert_eq!(experience_count, 1);
        let count: i64 = rullst::db::sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 6);
    }
}
