use leptos::prelude::*;

#[component]
pub fn App(database_configured: bool) -> impl IntoView {
    let status_text = if database_configured {
        "configured"
    } else {
        "not configured"
    };

    view! {
        <main style="font-family: sans-serif; max-width: 42rem; margin: 2rem auto; line-height: 1.4; padding: 0 1rem;">
            <h1>"blog001"</h1>
            <p>"Leptos SSR with Axum is initialized."</p>
            <p>{format!("DATABASE_URL is {status_text}.")}</p>
            <p>"Health check endpoint: /healthz"</p>
        </main>
    }
}
