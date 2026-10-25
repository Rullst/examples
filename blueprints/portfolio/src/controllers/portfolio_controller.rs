use crate::models::experience::Experience;
use crate::models::profile::Profile;
use crate::models::project::Project;
use crate::models::skill::Skill;
use crate::pages::home;
use rullst::server::IntoResponse;

pub async fn index(
    rullst::server::Extension(csrf_token): rullst::server::Extension<rullst::security::CsrfToken>,
) -> impl IntoResponse {
    let profile = Profile::query()
        .order_by("id")
        .first()
        .await
        .unwrap_or(None)
        .unwrap_or(Profile {
            id: 1,
            name: "Venelouis".to_string(),
            title: "Senior Rust & AI Engineer".to_string(),
            subtitle: "Specializing in hyper-concurrent web backends, Generative AI integration, and high-throughput Rust architectures.".to_string(),
            email: "officialrullst@gmail.com".to_string(),
            website: "https://rullst.win".to_string(),
            avatar_url: "/static/rullst.png".to_string(),
            github_url: "https://github.com/Rullst".to_string(),
            linkedin_url: "https://linkedin.com/company/rullst".to_string(),
            snapshot: rullst::db::RowSnapshot::default(),
        });
    let projects = Project::query()
        .order_by("id")
        .get()
        .await
        .unwrap_or_default();
    let experiences = Experience::query()
        .order_by("id")
        .get()
        .await
        .unwrap_or_default();
    let skills = Skill::query()
        .order_by("id")
        .get()
        .await
        .unwrap_or_default();

    home::render(
        &profile,
        &projects,
        &experiences,
        &skills,
        csrf_token.as_str(),
    )
}
