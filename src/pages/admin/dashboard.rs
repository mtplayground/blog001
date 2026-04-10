use leptos::{component, view, CollectView, IntoView};

#[derive(Clone, Debug)]
pub struct RecentPostSummary {
    pub title: String,
    pub slug: String,
    pub is_published: bool,
    pub updated_at: String,
}

#[component]
pub fn AdminDashboard(post_count: i64, recent_posts: Vec<RecentPostSummary>) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="grid gap-4 md:grid-cols-3">
                <section class="rounded-xl border border-emerald-100 bg-emerald-50/70 p-4">
                    <p class="text-xs font-semibold uppercase tracking-wide text-emerald-800">"Total Posts"</p>
                    <p class="mt-2 text-3xl font-bold text-emerald-900">{post_count}</p>
                </section>
                <a href="/admin/posts/new" class="rounded-xl border border-slate-200 bg-white p-4 transition hover:border-emerald-300 hover:bg-emerald-50/40">
                    <p class="text-sm font-semibold text-slate-900">"New Post"</p>
                    <p class="mt-1 text-sm text-slate-600">"Create a new draft or publish immediately."</p>
                </a>
                <a href="/admin/posts" class="rounded-xl border border-slate-200 bg-white p-4 transition hover:border-emerald-300 hover:bg-emerald-50/40">
                    <p class="text-sm font-semibold text-slate-900">"Manage Posts"</p>
                    <p class="mt-1 text-sm text-slate-600">"Edit, delete, or toggle publish status."</p>
                </a>
            </div>

            <section class="rounded-xl border border-slate-200 bg-white p-4">
                <div class="mb-3 flex items-center justify-between">
                    <h2 class="text-base font-semibold text-slate-900">"Recent Posts"</h2>
                    <a href="/admin/posts" class="text-sm font-medium text-emerald-700 hover:text-emerald-800">"View all"</a>
                </div>

                {if recent_posts.is_empty() {
                    view! {
                        <p class="text-sm text-slate-600">"No posts yet. Start by creating your first post."</p>
                    }.into_view()
                } else {
                    view! {
                        <ul class="space-y-3">
                            {recent_posts
                                .into_iter()
                                .map(|post| {
                                    let badge = if post.is_published { "Published" } else { "Draft" };
                                    let badge_class = if post.is_published {
                                        "bg-emerald-100 text-emerald-800"
                                    } else {
                                        "bg-amber-100 text-amber-800"
                                    };
                                    view! {
                                        <li class="rounded-lg border border-slate-100 px-3 py-2">
                                            <div class="flex items-center justify-between gap-3">
                                                <a href={format!("/admin/posts/{}", post.slug)} class="truncate text-sm font-medium text-slate-900 hover:text-emerald-800">
                                                    {post.title}
                                                </a>
                                                <span class={format!("inline-flex rounded-full px-2 py-0.5 text-xs font-semibold {badge_class}")}>
                                                    {badge}
                                                </span>
                                            </div>
                                            <p class="mt-1 text-xs text-slate-500">{format!("Last updated {}", post.updated_at)}</p>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }.into_view()
                }}
            </section>
        </div>
    }
}
