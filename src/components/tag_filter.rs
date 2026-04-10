use leptos::{component, view, CollectView, IntoView};

use crate::components::tag_badge::TagBadge;

#[derive(Clone, Debug)]
pub struct TagFilterItem {
    pub name: String,
    pub slug: String,
    pub count: usize,
}

#[component]
pub fn TagFilter(
    tags: Vec<TagFilterItem>,
    #[prop(optional)] selected_slug: Option<String>,
) -> impl IntoView {
    let selected_slug = selected_slug.unwrap_or_else(|| "all".to_string());

    let script = r#"
(() => {
  const root = document.getElementById('tag-filter-root');
  if (!root) {
    return;
  }

  const buttons = Array.from(root.querySelectorAll('[data-tag-button]'));
  const cards = Array.from(document.querySelectorAll('[data-post-card]'));

  const applySelection = (slug) => {
    buttons.forEach((button) => {
      const value = button.getAttribute('data-tag-slug') || 'all';
      const isSelected = value === slug;
      button.classList.toggle('border-emerald-300', isSelected);
      button.classList.toggle('bg-emerald-100', isSelected);
      button.classList.toggle('text-emerald-900', isSelected);
      button.classList.toggle('border-slate-200', !isSelected);
      button.classList.toggle('bg-white', !isSelected);
      button.classList.toggle('text-slate-700', !isSelected);
    });

    cards.forEach((card) => {
      const raw = card.getAttribute('data-post-tags') || '';
      const tags = raw.split(',').map((t) => t.trim()).filter(Boolean);
      const visible = slug === 'all' || tags.includes(slug);
      card.classList.toggle('hidden', !visible);
    });
  };

  buttons.forEach((button) => {
    button.addEventListener('click', () => {
      const slug = button.getAttribute('data-tag-slug') || 'all';
      applySelection(slug);
    });
  });

  const initial = root.getAttribute('data-selected-tag') || 'all';
  applySelection(initial);
})();
"#;

    view! {
        <aside id="tag-filter-root" data-selected-tag={selected_slug.clone()} class="space-y-3 rounded-xl border border-slate-200 bg-white p-4">
            <div class="flex items-center justify-between gap-3">
                <h2 class="text-sm font-semibold uppercase tracking-wide text-slate-700">"Filter by tag"</h2>
                <button
                    type="button"
                    data-tag-slug="all"
                    data-tag-button="true"
                    class="inline-flex items-center gap-2 rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-medium text-slate-700 transition hover:border-emerald-300 hover:bg-emerald-50"
                >
                    "All"
                </button>
            </div>

            <div class="flex flex-wrap gap-2">
                {tags
                    .into_iter()
                    .map(|tag| {
                        let is_selected = selected_slug == tag.slug;
                        view! {
                            <TagBadge
                                name=tag.name
                                slug=tag.slug
                                count=tag.count
                                selected=is_selected
                            />
                        }
                    })
                    .collect_view()}
            </div>

            <p class="text-xs text-slate-500">"Select a tag to filter visible posts."</p>
            <script>{script}</script>
        </aside>
    }
}
