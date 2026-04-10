use leptos::{component, view, CollectView, IntoView};

use crate::components::tag_badge::TagBadge;

#[derive(Clone, Debug)]
pub struct PostTag {
    pub name: String,
    pub slug: String,
}

#[component]
pub fn PostPage(
    title: String,
    published_at: String,
    tags: Vec<PostTag>,
    content_html: String,
) -> impl IntoView {
    view! {
        <article class="mx-auto w-full max-w-3xl space-y-6 rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
            <header class="space-y-3">
                <h1 class="text-3xl font-bold tracking-tight text-slate-900">{title}</h1>
                <p class="text-sm text-slate-500">{published_at}</p>
                <div class="flex flex-wrap gap-2">
                    {tags
                        .into_iter()
                        .map(|tag| {
                            view! {
                                <TagBadge name=tag.name slug=tag.slug />
                            }
                        })
                        .collect_view()}
                </div>
            </header>

            <div class="prose prose-slate max-w-none leading-7 text-slate-800" inner_html=content_html></div>
        </article>
    }
}
