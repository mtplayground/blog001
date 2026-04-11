use leptos::{component, view, Children, IntoView};

#[component]
pub fn AdminLayout(username: String, children: Children) -> impl IntoView {
    let script = r#"
(() => {
  const logout = document.getElementById('admin-logout');
  if (!logout) {
    return;
  }

  logout.addEventListener('click', async () => {
    logout.setAttribute('disabled', 'true');

    try {
      await fetch('/auth/logout', { method: 'POST' });
    } finally {
      window.location.assign('/login');
    }
  });
})();
"#;

    view! {
        <div class="shell flex min-h-screen flex-col">
            <header class="shell-header">
                <div class="mx-auto flex w-full max-w-6xl items-center justify-between px-4 py-4 sm:px-6 lg:px-8">
                    <div>
                        <a class="text-lg font-semibold tracking-tight text-brand-900 no-underline" href="/admin">
                            "Admin"
                        </a>
                        <p class="text-sm text-slate-600">{format!("Signed in as {username}")}</p>
                    </div>
                    <button
                        id="admin-logout"
                        type="button"
                        class="inline-flex items-center justify-center rounded-md border border-slate-300 bg-white px-3 py-2 text-sm font-medium text-slate-800 transition hover:bg-slate-100 disabled:cursor-not-allowed disabled:opacity-70"
                    >
                        "Logout"
                    </button>
                </div>
            </header>

            <main class="shell-main flex-1">
                <section class="card space-y-4">
                    {children()}
                </section>
            </main>

            <footer class="shell-footer mt-auto">
                <div class="mx-auto w-full max-w-6xl px-4 py-4 text-sm text-slate-600 sm:px-6 lg:px-8">
                    "Protected admin area"
                </div>
            </footer>
            <script inner_html=script></script>
        </div>
    }
}
