use leptos::prelude::*;

use crate::components::layout::BaseLayout;

#[component]
pub fn App(database_configured: bool) -> impl IntoView {
    let status_text = if database_configured {
        "configured"
    } else {
        "not configured"
    };

    view! {
        <BaseLayout title="Leptos SSR is ready">
            <p class="text-base text-slate-700">"Tailwind base layout has been configured for this project."</p>
            <p class="text-base text-slate-700">{format!("DATABASE_URL is {status_text}.")}</p>
            <p class="text-sm text-slate-500">"Health check endpoint: /healthz"</p>
        </BaseLayout>
    }
}
