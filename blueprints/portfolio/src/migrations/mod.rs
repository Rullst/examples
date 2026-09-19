pub mod m20260701000000_create_portfolio_tables;
pub mod m20260919000000_refresh_showcase;

pub fn get_migrations() -> Vec<Box<dyn rullst::db::schema::Migration>> {
    vec![
        Box::new(m20260701000000_create_portfolio_tables::CreatePortfolioTables),
        Box::new(m20260919000000_refresh_showcase::RefreshShowcase),
    ]
}
