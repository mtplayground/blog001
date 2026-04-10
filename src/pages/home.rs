use leptos::{component, view, CollectView, IntoView};

use crate::components::{post_card::PostCard, tag_filter::{TagFilter, TagFilterItem}};

#[derive(Clone, Debug)]
pub struct HomePostSummary {
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub published_at: String,
    pub tag_names: Vec<String>,
    pub tag_slugs: Vec<String>,
}

#[component]
pub fn HomePage(posts: Vec<HomePostSummary>, tags: Vec<TagFilterItem>) -> impl IntoView {
    view! {
        <HomePageWithSelection posts=posts tags=tags />
    }
}

#[component]
pub fn HomePageWithSelection(
    posts: Vec<HomePostSummary>,
    tags: Vec<TagFilterItem>,
    #[prop(optional)]
    selected_slug: Option<String>,
) -> impl IntoView {
    let selected_slug = selected_slug.unwrap_or_else(|| "all".to_string());

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold tracking-tight text-slate-900 sm:text-3xl">"Latest Posts"</h1>
                <p class="mt-1 text-sm text-slate-600">"Published posts in reverse chronological order."</p>
            </div>

            <TagFilter tags=tags selected_slug=selected_slug />

            {if posts.is_empty() {
                view! {
                    <p class="rounded-xl border border-slate-200 bg-white px-4 py-6 text-sm text-slate-600">"No published posts yet."</p>
                }.into_view()
            } else {
                view! {
                    <div class="grid gap-4">
                        {posts
                            .into_iter()
                            .map(|post| {
                                view! {
                                    <PostCard
                                        title=post.title
                                        slug=post.slug
                                        excerpt=post.excerpt
                                        published_at=post.published_at
                                        tag_names=post.tag_names
                                        tag_slugs=post.tag_slugs
                                    />
                                }
                            })
                            .collect_view()}
                    </div>
                }.into_view()
            }}
        </div>
    }
}
