use leptos::{component, view, IntoView};

use crate::components::layout::BaseLayout;
use crate::router;

#[component]
pub fn App(database_configured: bool) -> impl IntoView {
    let status_text = if database_configured {
        "configured"
    } else {
        "not configured"
    };
    let public_routes = router::public_routes();
    let admin_routes = router::admin_routes();

    view! {
        <BaseLayout title="Leptos SSR is ready">
            <p class="text-base text-slate-700">"Tailwind base layout has been configured for this project."</p>
            <p class="text-base text-slate-700">{format!("DATABASE_URL is {status_text}.")}</p>
            <p class="text-sm text-slate-500">"Health check endpoint: /healthz"</p>
            <div class="rounded-lg border border-slate-200 bg-white p-4">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-700">"Configured Public Routes"</h2>
                <ul class="mt-2 list-disc pl-5 text-sm text-slate-600">
                    <li>{public_routes[0]}</li>
                    <li>{public_routes[1]}</li>
                    <li>{public_routes[2]}</li>
                </ul>
            </div>
            <div class="rounded-lg border border-slate-200 bg-white p-4">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-700">"Configured Admin Routes"</h2>
                <ul class="mt-2 list-disc pl-5 text-sm text-slate-600">
                    <li>{admin_routes[0]}</li>
                    <li>{admin_routes[1]}</li>
                    <li>{admin_routes[2]}</li>
                    <li>{admin_routes[3]}</li>
                </ul>
            </div>
        </BaseLayout>
    }
}
