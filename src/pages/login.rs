use leptos::{component, view, IntoView};

use crate::components::layout::BaseLayout;

#[component]
pub fn LoginPage() -> impl IntoView {
    let script = r#"
(() => {
  const form = document.getElementById('login-form');
  const errorEl = document.getElementById('login-error');
  const submitBtn = document.getElementById('login-submit');

  if (!form || !errorEl || !submitBtn) {
    return;
  }

  form.addEventListener('submit', async (event) => {
    event.preventDefault();

    errorEl.textContent = '';
    submitBtn.setAttribute('disabled', 'true');

    const formData = new FormData(form);
    const username = String(formData.get('username') ?? '').trim();
    const password = String(formData.get('password') ?? '');

    if (!username || !password) {
      errorEl.textContent = 'Username and password are required.';
      submitBtn.removeAttribute('disabled');
      return;
    }

    try {
      const response = await fetch('/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username, password })
      });

      const data = await response.json().catch(() => ({}));

      if (!response.ok || !data.authenticated) {
        errorEl.textContent = 'Invalid username or password.';
        submitBtn.removeAttribute('disabled');
        return;
      }

      window.location.assign('/');
    } catch (_) {
      errorEl.textContent = 'Login failed due to a network error. Please try again.';
      submitBtn.removeAttribute('disabled');
    }
  });
})();
"#;

    view! {
        <BaseLayout title="Sign In">
            <div class="mx-auto w-full max-w-md">
                <form id="login-form" class="space-y-4 rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
                    <div>
                        <label class="mb-1 block text-sm font-medium text-slate-700" for="username">"Username"</label>
                        <input
                            class="w-full rounded-md border border-slate-300 px-3 py-2 text-slate-900 outline-none ring-emerald-200 transition focus:border-emerald-500 focus:ring"
                            id="username"
                            name="username"
                            type="text"
                            autocomplete="username"
                            required=true
                        />
                    </div>

                    <div>
                        <label class="mb-1 block text-sm font-medium text-slate-700" for="password">"Password"</label>
                        <input
                            class="w-full rounded-md border border-slate-300 px-3 py-2 text-slate-900 outline-none ring-emerald-200 transition focus:border-emerald-500 focus:ring"
                            id="password"
                            name="password"
                            type="password"
                            autocomplete="current-password"
                            required=true
                        />
                    </div>

                    <p id="login-error" class="min-h-5 text-sm font-medium text-rose-600"></p>

                    <button
                        id="login-submit"
                        type="submit"
                        class="inline-flex w-full items-center justify-center rounded-md bg-emerald-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-emerald-700 disabled:cursor-not-allowed disabled:bg-emerald-400"
                    >
                        "Sign In"
                    </button>
                </form>
            </div>
            <script>{script}</script>
        </BaseLayout>
    }
}
