use crate::models::category::Category;
use crate::models::course::Course;
use rullst::response::Html;
use rullst::server::IntoResponse;

#[derive(serde::Deserialize)]
pub struct ChatPayload {
    pub cloud_ai: Option<String>,
    pub message: String,
}

fn is_portuguese(text: &str) -> bool {
    let lower = text.to_lowercase();
    let pt_markers = [
        "você",
        "voce",
        "quais",
        "qual",
        "como",
        "onde",
        "porque",
        "por que",
        "habilidade",
        "habilidades",
        "projeto",
        "projetos",
        "trabalho",
        "carreira",
        "experiência",
        "experiencia",
        "contato",
        "gosta",
        "gosto",
        "olá",
        "ola",
        "bom dia",
        "boa tarde",
        "boa noite",
        "ajuda",
        "curso",
        "cursos",
        "aula",
        "aulas",
        "trilha",
        "trilhas",
        "aluno",
        "estudante",
        "ensine",
        "explique",
        "o que",
        "quero",
        "preciso",
        "meu",
        "minha",
        "nosso",
        "nossa",
        "linguagem",
        "programação",
        "aprender",
        "aprenda",
        "me diga",
        "me explique",
        "é um",
        "é uma",
        "são",
        "tem",
        "têm",
        "consegue",
        "funciona",
    ];
    pt_markers.iter().any(|&m| lower.contains(m))
}

fn fallback_offline_response(
    user_msg: &str,
    courses: &[Course],
    categories: &[Category],
) -> String {
    let lower = user_msg.to_lowercase();
    let pt = is_portuguese(&lower);

    // Portuguese responses ONLY when the user explicitly asked in Portuguese AND there is a repertoire match
    if pt {
        if lower.contains("pass")
            || lower.contains("senha")
            || lower.contains("admin")
            || lower.contains("nexus")
            || lower.contains("studio")
            || lower.contains("credencial")
        {
            return "<p>🛡️ <strong>Rullst AI Guardrail:</strong> Por diretrizes estritas de segurança Zero-Trust, credenciais administrativas e senhas do Nexus CMS e Studio Cockpit não são gerenciadas nem reveladas pelo Copilot Acadêmico.</p>\
                    <p style=\"font-size: 0.82rem; color: #a1a1aa; margin-top: 0.5rem;\">Para assistir às aulas e explorar a plataforma como estudante, utilize a conta de demonstração disponibilizada na tela de login (<code>/login</code>).</p>".to_string();
        }

        if lower.contains("rullst") {
            return format!(
                "<p><strong>Rullst</strong> é um ecossistema full-stack moderno em Rust focado em ultra-alta performance, concorrência assíncrona e arquitetura <em>Zero-Bundle</em>:</p>\
                 <ul style=\"margin: 0.5rem 0; padding-left: 1.25rem; font-size: 0.9rem; line-height: 1.6;\">\
                   <li>⚡ <strong>SSR Declarativo:</strong> Renderização ultra rápida no servidor com a macro <code>html!</code> sem sobrecarga de runtime de JavaScript.</li>\
                   <li>🌐 <strong>HTMX Nativo:</strong> Interatividade dinâmica e reativa diretamente em HTML sem necessidade de SPAs pesadas.</li>\
                   <li>💾 <strong>Banco de Dados & ORM:</strong> Suporte assíncrono para SQLite e PostgreSQL com migrações tipadas e consultas de alta velocidade.</li>\
                   <li>🛡️ <strong>Sovereign AI Guardrails:</strong> Firewall nativo de segurança contra prompt injections, vazamentos e jailbreaks.</li>\
                   <li>🎛️ <strong>Nexus & Studio:</strong> Módulos integrados de CMS headless e cockpit de desenvolvimento para produtividade total.</li>\
                 </ul>\
                 <p style=\"font-size: 0.8rem; color: #34d399; margin-top: 0.6rem;\"><em>Dica: O Rullst permite que você construa aplicações completas em Rust com latência de resposta na casa dos microssegundos!</em></p>"
            );
        }

        if lower.contains("rust")
            || lower.contains("memory")
            || lower.contains("ownership")
            || lower.contains("borrow")
            || lower.contains("lifetime")
            || lower.contains("tokio")
            || lower.contains("async")
            || lower.contains("concorr")
            || lower.contains("arc")
            || lower.contains("mutex")
            || lower.contains("smart pointer")
        {
            return format!(
                "<p><strong>Rust</strong> é uma linguagem de sistemas focada em segurança, velocidade e concorrência sem depender de garbage collector:</p>\
                 <ul style=\"margin: 0.5rem 0; padding-left: 1.25rem; font-size: 0.9rem; line-height: 1.6;\">\
                   <li>🦀 <strong>Ownership & Borrowing:</strong> Cada valor na memória tem um dono exclusivo. Você pode emprestar referências imutáveis (<code>&T</code>) ou uma única referência mutável (<code>&mut T</code>), evitando data races em compilação.</li>\
                   <li>⏳ <strong>Lifetimes:</strong> O Borrow Checker garante matematicamente que nenhuma referência aponte para memória já desalocada (prevenindo <em>dangling pointers</em>).</li>\
                   <li>⚡ <strong>Tokio & Async:</strong> Concorrência cooperativa baseada em polling de <code>Future</code>, permitindo que um único servidor manipule centenas de milhares de conexões simultâneas.</li>\
                   <li>🧠 <strong>Smart Pointers:</strong> <code>Box<T></code> para heap, <code>Arc<T></code> para compartilhamento atômico seguro entre threads e <code>Mutex<T></code> / <code>RwLock<T></code> para sincronização.</li>\
                 </ul>\
                 <p style=\"font-size: 0.8rem; color: #34d399; margin-top: 0.6rem;\"><em>Você pode aprofundar esses conceitos explorando os módulos e aulas do curso 'Rust Web Systems' na nossa plataforma!</em></p>"
            );
        }

        if lower.contains("curso")
            || lower.contains("course")
            || lower.contains("catalog")
            || lower.contains("aula")
            || lower.contains("lesson")
            || lower.contains("trilha")
        {
            let mut course_list = String::new();
            for c in courses.iter().take(5) {
                course_list.push_str(&format!(
                    "<li><strong>{}</strong>: {}</li>",
                    rullst::html::escape_str(&c.title),
                    rullst::html::escape_str(&c.description)
                ));
            }
            return format!(
                "<p>Aqui estão alguns dos cursos ativos na <strong>Rullst Academy</strong>:</p>\
                 <ul style=\"margin: 0.5rem 0; padding-left: 1.25rem; font-size: 0.9rem; line-height: 1.6;\">{}</ul>\
                 <p style=\"font-size: 0.8rem; color: #34d399; margin-top: 0.6rem;\">Cada curso inclui aulas práticas, código-fonte para download e exercícios de fixação.</p>",
                if course_list.is_empty() {
                    "<li>Curso de Engenharia de Sistemas em Rust e Rullst Web</li>".to_string()
                } else {
                    course_list
                }
            );
        }
        // If user asked in Portuguese but there is no specific repertoire match, fall through to default English response!
    }

    // DEFAULT LANGUAGE: ENGLISH
    if lower.contains("pass")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("admin")
        || lower.contains("nexus")
        || lower.contains("studio")
        || lower.contains("credential")
    {
        return "<p>🛡️ <strong>Rullst AI Guardrail:</strong> Under strict Zero-Trust security policies, administrative credentials and default passwords for Nexus CMS and Studio Cockpit are never managed or disclosed by the Academic Copilot.</p>\
                <p style=\"font-size: 0.82rem; color: #a1a1aa; margin-top: 0.5rem;\">To attend lessons and explore the platform as a student, use the public demo account provided on the login page (<code>/login</code>).</p>".to_string();
    }

    if lower.contains("rullst") {
        return format!(
            "<p><strong>Rullst</strong> is a modern full-stack web ecosystem in Rust focused on hyper-concurrency, microsecond latency, and a <em>Zero-Bundle</em> architecture:</p>\
             <ul style=\"margin: 0.5rem 0; padding-left: 1.25rem; font-size: 0.9rem; line-height: 1.6;\">\
               <li>⚡ <strong>Declarative SSR:</strong> Blazing-fast server-side rendering with the compile-time <code>html!</code> macro without JavaScript runtime overhead.</li>\
               <li>🌐 <strong>Native HTMX:</strong> Dynamic and reactive user interfaces directly in HTML without bloated client SPAs.</li>\
               <li>💾 <strong>Database & Async ORM:</strong> Strongly-typed SQLite and PostgreSQL integration with compile-time migrations.</li>\
               <li>🛡️ <strong>Sovereign AI Guardrails:</strong> Built-in firewall heuristics preventing prompt injections, exfiltration, and jailbreaks.</li>\
               <li>🎛️ <strong>Nexus & Studio:</strong> Embedded headless CMS and development cockpit for maximum developer productivity.</li>\
             </ul>\
             <p style=\"font-size: 0.8rem; color: #34d399; margin-top: 0.6rem;\"><em>Tip: Rullst allows you to ship full web applications in Rust with sub-millisecond response times!</em></p>"
        );
    }

    if lower.contains("rust")
        || lower.contains("memory")
        || lower.contains("ownership")
        || lower.contains("borrow")
        || lower.contains("lifetime")
        || lower.contains("tokio")
        || lower.contains("async")
        || lower.contains("concurr")
        || lower.contains("arc")
        || lower.contains("mutex")
        || lower.contains("smart pointer")
    {
        return format!(
            "<p><strong>Rust</strong> is a systems programming language delivering memory safety, thread safety, and blazing performance without a garbage collector:</p>\
             <ul style=\"margin: 0.5rem 0; padding-left: 1.25rem; font-size: 0.9rem; line-height: 1.6;\">\
               <li>🦀 <strong>Ownership & Borrowing:</strong> Every value in memory has a single owner. You can lend multiple immutable references (<code>&T</code>) or a single mutable reference (<code>&mut T</code>), preventing data races at compile time.</li>\
               <li>⏳ <strong>Lifetimes:</strong> The Borrow Checker mathematically proves references will never outlive the underlying memory (preventing dangling pointers).</li>\
               <li>⚡ <strong>Tokio & Async:</strong> Cooperative event-driven concurrency powered by non-blocking <code>Future</code> polling, handling millions of simultaneous connections with tiny memory footprints.</li>\
               <li>🧠 <strong>Smart Pointers:</strong> <code>Box<T></code> for heap allocations, <code>Arc<T></code> for safe multi-thread atomic reference counting, and <code>Mutex<T></code> / <code>RwLock<T></code> for thread synchronization.</li>\
             </ul>\
             <p style=\"font-size: 0.8rem; color: #34d399; margin-top: 0.6rem;\"><em>You can dive into these principles through the hands-on lessons in our 'Rust Web Systems' course!</em></p>"
        );
    }

    if lower.contains("course")
        || lower.contains("catalog")
        || lower.contains("lesson")
        || lower.contains("curriculum")
        || lower.contains("track")
    {
        let mut course_list = String::new();
        for c in courses.iter().take(5) {
            course_list.push_str(&format!(
                "<li><strong>{}</strong>: {}</li>",
                rullst::html::escape_str(&c.title),
                rullst::html::escape_str(&c.description)
            ));
        }
        format!(
            "<p>Here are highlighted active courses available in the <strong>Rullst LMS Showcase</strong>:</p>\
             <ul style=\"margin: 0.5rem 0; padding-left: 1.25rem; font-size: 0.9rem; line-height: 1.6;\">{}</ul>\
             <p style=\"font-size: 0.8rem; color: #34d399; margin-top: 0.6rem;\">Every course features hands-on lessons, downloadable code, and interactive quizzes.</p>",
            if course_list.is_empty() {
                "<li>Rust Web Systems and Rullst Full-Stack Engineering</li>".to_string()
            } else {
                course_list
            }
        )
    } else {
        let cat_names = categories
            .iter()
            .map(|c| rullst::html::escape_str(&c.name))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "<p>Hello! I am the <strong>Academic Copilot</strong> for the Rullst LMS Showcase, your expert tutor in <strong>Rust</strong> and the <strong>Rullst</strong> ecosystem! 🎓✨</p>\
             <p style=\"margin-top: 0.5rem;\">I am here to answer any questions you have regarding:</p>\
             <ul style=\"margin: 0.5rem 0; padding-left: 1.25rem; font-size: 0.88rem; line-height: 1.5;\">\
               <li>The Rust language (ownership, borrowing, lifetimes, Tokio async, smart pointers, performance)</li>\
               <li>The Rullst framework (Zero-Bundle architecture, SSR with <code>html!</code>, HTMX, and typed ORM)</li>\
               <li>Active courses and learning tracks in: <strong>{}</strong></li>\
             </ul>\
             <p style=\"font-size: 0.76rem; color: #a1a1aa; margin-top: 0.75rem;\">⚡ <em>Tip: Ask 'What is Rullst?', 'How does ownership work in Rust?' or 'What courses are available?'.</em></p>",
            if cat_names.is_empty() {
                "Systems and Web Rust"
            } else {
                &cat_names
            }
        )
    }
}

fn cloud_requested(payload: &ChatPayload) -> bool {
    payload.cloud_ai.as_deref() == Some("yes")
}

pub async fn chat(
    rullst::server::Form(payload): rullst::server::Form<ChatPayload>,
) -> impl IntoResponse {
    let raw_msg = payload.message.trim();
    if raw_msg.is_empty() || raw_msg.chars().count() > 600 {
        return Html(
            "<p>Please enter a question of 1–600 characters. Do not include personal data.</p>"
                .to_string(),
        )
        .into_response();
    }
    let courses = Course::all().await.unwrap_or_default();
    let categories = Category::all().await.unwrap_or_default();
    let local = || fallback_offline_response(raw_msg, &courses, &categories);
    let key = if cloud_requested(&payload) {
        std::env::var("GROQ_API_KEY")
            .or_else(|_| std::env::var("GROQ_KEY"))
            .or_else(|_| std::env::var("GROQ_APIKEY"))
            .or_else(|_| std::env::var("GROQ_TOKEN"))
            .ok()
            .map(|value| value.trim().trim_matches(['\"', '\'']).to_string())
            .filter(|value| !value.is_empty() && !value.starts_with("mock_"))
    } else {
        None
    };
    let mut cloud_reply = false;
    let content = if let Some(key) = key {
        let base_url = std::env::var("GROQ_BASE_URL")
            .unwrap_or_else(|_| "https://api.groq.com/openai/v1".into());
        let model = std::env::var("GROQ_MODEL").unwrap_or_else(|_| "openai/gpt-oss-120b".into());
        // No database records, account identifiers or learning history go to the provider.
        let prompt = "You are a tutor in a simple Rullst v12 LMS showcase. Explain Rust and Rullst clearly in English, or Portuguese if requested. The real Academy is at https://academy.rullst.win. The homepage shows the shared demo login for platform, Nexus and Studio. Never guess private credentials. Do not claim to know a visitor's learning history or unpublished course data.";
        match rullst::ai::providers::openai_compatible::OpenAiCompatibleProvider::try_cloud(
            base_url, key, model,
        ) {
            Ok(provider) => {
                let client = rullst::ai::AiClient::new(provider);
                match client.chat().system(prompt).user(raw_msg).send().await {
                    Ok(reply) => {
                        cloud_reply = true;
                        rullst::html::escape_str(&reply).replace('\n', "<br>")
                    }
                    Err(_) => {
                        tracing::warn!("LMS cloud response unavailable; returning a local reply");
                        local()
                    }
                }
            }
            Err(_) => local(),
        }
    } else {
        local()
    };
    let mode = if cloud_reply {
        "Cloud reply · Groq · Enabled for this request"
    } else {
        "Local reply · No external AI response used"
    };
    Html(format!("<div class=\"chat-bubble chat-bubble-assistant\"><div class=\"chat-bubble-sender\">Academic Copilot</div><div class=\"chat-bubble-body\">{content}</div><small>{mode}</small></div>")).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_requires_explicit_affirmative_choice() {
        for choice in [None, Some("no"), Some("true"), Some("")] {
            assert!(!cloud_requested(&ChatPayload {
                message: "hello".into(),
                cloud_ai: choice.map(str::to_owned)
            }));
        }
        assert!(cloud_requested(&ChatPayload {
            message: "hello".into(),
            cloud_ai: Some("yes".into())
        }));
    }

    #[test]
    fn test_offline_response_defaults_to_english() {
        let resp = fallback_offline_response("what is Rullst?", &[], &[]);
        assert!(resp.contains("is a modern full-stack web ecosystem in Rust"));
        assert!(!resp.contains("ecossistema full-stack"));

        let resp_rust =
            fallback_offline_response("explain ownership and memory safety in rust", &[], &[]);
        assert!(resp_rust.contains("Ownership & Borrowing"));
        assert!(!resp_rust.contains("Cada valor na memória"));
    }

    #[test]
    fn test_offline_response_answers_portuguese_only_when_requested_in_portuguese() {
        let resp = fallback_offline_response("o que é o Rullst?", &[], &[]);
        assert!(resp.contains("ecossistema full-stack moderno em Rust"));

        let resp_rust =
            fallback_offline_response("explique ownership e concorrência em rust", &[], &[]);
        assert!(resp_rust.contains("Cada valor na memória tem um dono exclusivo"));
    }

    #[test]
    fn test_guardrails_do_not_block_pedagogical_questions() {
        use rullst::ai::guardrails::AiGuardrails;

        let query = "what is Rullst?";
        let report = AiGuardrails::inspect(query);
        assert!(
            report.passed_heuristics(),
            "Normal question 'what is Rullst?' must not be blocked!"
        );

        let query_rust = "How does ownership and borrowing work in Rust?";
        let report_rust = AiGuardrails::inspect(query_rust);
        assert!(
            report_rust.passed_heuristics(),
            "Rust ownership question must not be blocked!"
        );
    }
}
