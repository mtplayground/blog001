use leptos::{component, view, CollectView, IntoView};

#[derive(Clone, Debug)]
pub struct AdminPostListItem {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub is_published: bool,
    pub updated_at: String,
}

#[component]
pub fn AdminPostsPage(posts: Vec<AdminPostListItem>) -> impl IntoView {
    let script = r#"
(() => {
  const status = document.getElementById('admin-posts-status');
  const buttons = Array.from(document.querySelectorAll('[data-delete-post]'));

  const setStatus = (message, isError = false) => {
    if (!status) {
      return;
    }
    status.textContent = message;
    status.className = isError ? 'text-sm font-medium text-rose-600' : 'text-sm font-medium text-emerald-700';
  };

  buttons.forEach((button) => {
    button.addEventListener('click', async () => {
      const id = button.getAttribute('data-post-id');
      const title = button.getAttribute('data-post-title') || 'this post';
      if (!id) {
        return;
      }

      const confirmed = window.confirm(`Delete ${title}? This action cannot be undone.`);
      if (!confirmed) {
        return;
      }

      button.setAttribute('disabled', 'true');
      setStatus('Deleting post...');

      try {
        const response = await fetch(`/server/posts/${id}`, { method: 'DELETE' });
        if (!response.ok && response.status !== 204) {
          const data = await response.json().catch(() => ({}));
          throw new Error(data.error || 'Failed to delete post.');
        }

        const row = button.closest('[data-post-row]');
        if (row) {
          row.remove();
        }
        setStatus('Post deleted.');
      } catch (error) {
        setStatus(error instanceof Error ? error.message : 'Failed to delete post.', true);
        button.removeAttribute('disabled');
      }
    });
  });
})();
"#;

    view! {
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h1 class="text-2xl font-bold tracking-tight text-slate-900 sm:text-3xl">"Manage Posts"</h1>
                    <p class="mt-1 text-sm text-slate-600">"All drafts and published posts are listed below."</p>
                </div>
                <a
                    href="/admin/posts/new"
                    class="inline-flex items-center justify-center rounded-md bg-emerald-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-emerald-700"
                >
                    "New Post"
                </a>
            </div>

            {if posts.is_empty() {
                view! {
                    <p class="rounded-lg border border-slate-200 bg-white px-4 py-6 text-sm text-slate-600">
                        "No posts found yet. Create your first post to get started."
                    </p>
                }.into_view()
            } else {
                view! {
                    <div class="overflow-hidden rounded-xl border border-slate-200 bg-white">
                        <table class="min-w-full divide-y divide-slate-200 text-sm">
                            <thead class="bg-slate-50 text-left text-xs font-semibold uppercase tracking-wide text-slate-600">
                                <tr>
                                    <th class="px-4 py-3">"Title"</th>
                                    <th class="px-4 py-3">"Status"</th>
                                    <th class="px-4 py-3">"Updated"</th>
                                    <th class="px-4 py-3 text-right">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-slate-100">
                                {posts
                                    .into_iter()
                                    .map(|post| {
                                        let status = if post.is_published { "Published" } else { "Draft" };
                                        let status_class = if post.is_published {
                                            "bg-emerald-100 text-emerald-800"
                                        } else {
                                            "bg-amber-100 text-amber-800"
                                        };

                                        view! {
                                            <tr data-post-row="true" class="hover:bg-slate-50/70">
                                                <td class="px-4 py-3">
                                                    <p class="font-medium text-slate-900">{post.title.clone()}</p>
                                                    <p class="text-xs text-slate-500">{format!("/{}", post.slug)}</p>
                                                </td>
                                                <td class="px-4 py-3">
                                                    <span class={format!("inline-flex rounded-full px-2 py-0.5 text-xs font-semibold {status_class}")}>
                                                        {status}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-slate-600">{post.updated_at.clone()}</td>
                                                <td class="px-4 py-3">
                                                    <div class="flex items-center justify-end gap-2">
                                                        <a
                                                            href={format!("/admin/posts/{}/edit", post.id)}
                                                            class="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 transition hover:bg-slate-100"
                                                        >
                                                            "Edit"
                                                        </a>
                                                        <button
                                                            type="button"
                                                            data-delete-post="true"
                                                            data-post-id={post.id.to_string()}
                                                            data-post-title={post.title.clone()}
                                                            class="rounded-md border border-rose-200 px-3 py-1.5 text-xs font-medium text-rose-700 transition hover:bg-rose-50"
                                                        >
                                                            "Delete"
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_view()
            }}

            <p id="admin-posts-status" class="text-sm font-medium text-slate-600"></p>
            <script>{script}</script>
        </div>
    }
}
