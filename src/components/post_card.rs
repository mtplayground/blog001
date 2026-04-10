use leptos::{component, view, CollectView, IntoView};

use crate::components::tag_badge::TagBadge;

#[component]
pub fn PostCard(
    title: String,
    slug: String,
    excerpt: String,
    published_at: String,
    tag_names: Vec<String>,
    tag_slugs: Vec<String>,
) -> impl IntoView {
    let filter_tags = tag_slugs.join(",");

    view! {
        <article data-post-card="true" data-post-tags={filter_tags} class="rounded-xl border border-slate-200 bg-white p-5 shadow-sm transition hover:border-emerald-200 hover:shadow">
            <div class="flex items-center justify-between gap-3">
                <a href={format!("/posts/{slug}")} class="text-lg font-semibold text-slate-900 transition hover:text-emerald-800">
                    {title}
                </a>
                <span class="text-xs text-slate-500">{published_at}</span>
            </div>

            <p class="mt-3 text-sm text-slate-700">{excerpt}</p>

            <div class="mt-4 flex flex-wrap gap-2">
                {tag_names
                    .into_iter()
                    .zip(tag_slugs.into_iter())
                    .map(|(name, slug)| {
                        view! {
                            <TagBadge name=name slug=slug />
                        }
                    })
                    .collect_view()}
            </div>
        </article>
    }
}
