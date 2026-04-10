use leptos::{component, view, Children, IntoView};

#[component]
pub fn BaseLayout(title: &'static str, children: Children) -> impl IntoView {
    let year = 2026;

    view! {
        <div class="shell flex flex-col">
            <header class="shell-header">
                <div class="mx-auto flex w-full max-w-5xl items-center justify-between px-4 py-4 sm:px-6 lg:px-8">
                    <a class="text-lg font-semibold tracking-tight text-brand-900 no-underline" href="/">
                        "blog001"
                    </a>
                    <p class="text-sm text-slate-600">"Leptos + Axum"</p>
                </div>
            </header>

            <main class="shell-main flex-1">
                <section class="card space-y-4">
                    <h1 class="text-2xl font-bold tracking-tight text-slate-900 sm:text-3xl">{title}</h1>
                    {children()}
                </section>
            </main>

            <footer class="shell-footer mt-auto">
                <div class="mx-auto flex w-full max-w-5xl items-center justify-between px-4 py-4 text-sm text-slate-600 sm:px-6 lg:px-8">
                    <span>{format!("© {year} blog001")}</span>
                    <a href="/healthz">"Health"</a>
                </div>
            </footer>
        </div>
    }
}
