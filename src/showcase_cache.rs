//! The template demo and Studio inspect the same process-local Rullst cache.

use axum::{
    Extension,
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use rullst::{
    Cache,
    cache::{CacheError, CacheInspection, MAX_CACHE_INSPECTION_ENTRIES},
    html,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[derive(Clone)]
pub struct ShowcaseCache {
    cache: Cache,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl Default for ShowcaseCache {
    fn default() -> Self {
        Self {
            cache: Cache::memory(),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl ShowcaseCache {
    pub async fn get(&self, key: &str) -> Result<Option<Arc<String>>, CacheError> {
        let value = self.cache.get(key).await?;
        let counter = if value.is_some() {
            &self.hits
        } else {
            &self.misses
        };
        counter.fetch_add(1, Ordering::Relaxed);
        Ok(value)
    }

    pub async fn put(&self, key: &str, value: &str, ttl: u64) -> Result<(), CacheError> {
        self.cache.put(key, value, Some(ttl)).await
    }

    async fn render(&self) -> String {
        match self.cache.inspect(MAX_CACHE_INSPECTION_ENTRIES).await {
            Ok(snapshot) => render_snapshot(
                &snapshot,
                self.hits.load(Ordering::Relaxed),
                self.misses.load(Ordering::Relaxed),
            ),
            Err(_) => html! {
                <p role="status">"Cache metrics are temporarily unavailable. Refresh to try again."</p>
            },
        }
    }
}

fn render_snapshot(snapshot: &CacheInspection, hits: u64, misses: u64) -> String {
    let lower_bound = if snapshot.truncated() { "≥ " } else { "" };
    let entries = format!("{lower_bound}{}", snapshot.entries().len());
    let bytes: usize = snapshot
        .entries()
        .iter()
        .map(|entry| entry.value_bytes())
        .sum();
    let size = format!("{lower_bound}{bytes} bytes");
    let lookups = hits.saturating_add(misses);
    let hit_rate = if lookups == 0 {
        "—".to_string()
    } else {
        format!("{:.1}%", hits as f64 / lookups as f64 * 100.0)
    };
    let lookup_detail = if lookups == 0 {
        "No cache lookups yet".to_string()
    } else {
        format!("{hits} hits · {misses} misses since startup")
    };
    let rows: String = snapshot
        .entries()
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let label = format!("Entry {}", index + 1);
            let bytes = format!("{} bytes", entry.value_bytes());
            let ttl = entry
                .remaining_ttl_ms()
                .map(|ms| format!("{:.1} s", ms as f64 / 1000.0))
                .unwrap_or_else(|| "No expiry".to_string());
            html! {
                <tr><td>{label}</td><td>{bytes}</td><td>{ttl}</td></tr>
            }
        })
        .collect();
    let table_content = if rows.is_empty() {
        html! { <tr><td colspan="3">"No live cache entries. Open the template demo to populate the cache."</td></tr> }
    } else {
        rows
    };
    let snapshot_note = if snapshot.truncated() {
        "Showing the first 200 entries; counts and sizes are lower bounds."
    } else {
        "Complete snapshot of live entries. Expired entries are excluded."
    };

    html! {
        <div id="showcase-cache-inspector" class="showcase-cache-inspector"
             hx-get="/studio/cache" hx-trigger="every 5s" hx-target="this" hx-swap="outerHTML">
            <style>{ rullst::html::RawHtml(include_str!("../static/showcase-cache.css").to_string()) }</style>
            <div class="showcase-cache-heading">
                <div>
                    <h1>"🧊 Studio Cache Inspector"</h1>
                    <p>"Live template rendering cache · In-memory Rullst Cache · Refreshes every 5 seconds"</p>
                </div>
                <a href="/templates-demo" target="_blank" rel="noopener noreferrer">"Open template demo ↗"</a>
            </div>
            <div class="showcase-cache-stats">
                <div class="showcase-cache-card">
                    <h2>"Active entries"</h2>
                    <strong data-cache-metric="entries">{entries}</strong>
                    <p>"Unexpired cached templates"</p>
                </div>
                <div class="showcase-cache-card">
                    <h2>"Hit rate"</h2>
                    <strong data-cache-metric="hit-rate">{hit_rate}</strong>
                    <p>{lookup_detail}</p>
                </div>
                <div class="showcase-cache-card">
                    <h2>"Cached value size"</h2>
                    <strong data-cache-metric="value-bytes">{size}</strong>
                    <p>"UTF-8 payload bytes; excludes cache overhead"</p>
                </div>
            </div>
            <div class="showcase-cache-table">
                <table>
                    <thead><tr><th>"Entry"</th><th>"Value size"</th><th>"Remaining TTL"</th></tr></thead>
                    <tbody>{ rullst::html::RawHtml(table_content) }</tbody>
                </table>
            </div>
            <p>{snapshot_note}</p>
            <p>"The template demo caches shared HTML for 60 seconds. Visit it twice to observe a miss followed by a hit. Metrics belong to this server process and reset on restart."</p>
        </div>
    }
}

pub async fn studio_cache_handler(
    Extension(cache): Extension<ShowcaseCache>,
    headers: HeaderMap,
) -> Response {
    let content = cache.render().await;
    let html = if headers.contains_key("hx-request") {
        content
    } else {
        rullst_studio::data_browser::studio_layout(content, None, &[]).replace(
            "</head>",
            "<script src=\"/static/htmx.js\" defer></script></head>",
        )
    };
    (
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Html(html),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inspection_measures_real_traffic_without_counting_itself() {
        let cache = ShowcaseCache::default();
        let empty = cache.render().await;
        assert!(empty.contains("No cache lookups yet"));
        assert!(cache.get("private:key").await.unwrap().is_none());
        cache.put("private:key", "secret-🦀", 60).await.unwrap();
        assert_eq!(
            cache.get("private:key").await.unwrap().unwrap().as_str(),
            "secret-🦀"
        );

        let html = cache.render().await;
        assert!(html.contains("50.0%"));
        assert!(html.contains("1 hits · 1 misses"));
        assert!(html.contains("11 bytes"));
        assert!(!html.contains("private:key"));
        assert!(!html.contains("secret-🦀"));
        assert!(cache.render().await.contains("1 hits · 1 misses"));
    }

    #[tokio::test]
    async fn expired_values_are_excluded_and_count_as_misses() {
        let cache = ShowcaseCache::default();
        cache.put("expired", "payload", 0).await.unwrap();
        assert!(cache.get("expired").await.unwrap().is_none());
        let html = cache.render().await;
        assert!(html.contains("No live cache entries"));
        assert!(html.contains("0 bytes"));
        assert!(html.contains("0.0%"));
    }

    #[tokio::test]
    async fn template_requests_share_cache_without_reusing_tenant_navigation() {
        let cache = ShowcaseCache::default();
        for tenant in ["tenant-startup", "tenant-enterprise"] {
            let page = rullst::multitenant::TENANT_CONTEXT
                .scope(
                    std::cell::RefCell::new(Some(tenant.to_string())),
                    crate::templates_demo::render_templates_demo_page(Extension(cache.clone())),
                )
                .await
                .0;
            assert!(page.contains(&format!("<strong>{tenant}</strong>")));
            assert!(!page.contains("{{ nav_html"));
            assert!(!page.contains("{{ footer_html"));
        }
        let inspection = cache.render().await;
        assert!(inspection.contains("1 hits · 1 misses"));
        assert!(inspection.contains("50.0%"));

        let full = studio_cache_handler(Extension(cache.clone()), HeaderMap::new()).await;
        assert_eq!(
            full.headers()[axum::http::header::CACHE_CONTROL],
            "no-store"
        );
        let full = axum::body::to_bytes(full.into_body(), usize::MAX)
            .await
            .unwrap();
        let full = String::from_utf8(full.to_vec()).unwrap();
        assert!(full.contains("<html"));
        assert!(full.contains("/static/htmx.js"));

        let mut headers = HeaderMap::new();
        headers.insert("hx-request", "true".parse().unwrap());
        let partial = studio_cache_handler(Extension(cache.clone()), headers).await;
        let partial = axum::body::to_bytes(partial.into_body(), usize::MAX)
            .await
            .unwrap();
        let partial = String::from_utf8(partial.to_vec()).unwrap();
        assert!(partial.contains("showcase-cache-inspector"));
        assert!(!partial.contains("<html"));
        assert!(cache.render().await.contains("1 hits · 1 misses"));
    }
}
