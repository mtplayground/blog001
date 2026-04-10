use leptos::{component, view, IntoView};

#[component]
pub fn TagBadge(
    name: String,
    slug: String,
    #[prop(optional)] count: Option<usize>,
    #[prop(optional)] selected: Option<bool>,
) -> impl IntoView {
    let is_selected = selected.unwrap_or(false);
    let badge_class = if is_selected {
        "inline-flex items-center gap-2 rounded-full border border-emerald-300 bg-emerald-100 px-3 py-1 text-xs font-semibold text-emerald-900 transition"
    } else {
        "inline-flex items-center gap-2 rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-medium text-slate-700 transition hover:border-emerald-300 hover:bg-emerald-50"
    };

    view! {
        <button
            type="button"
            class={badge_class}
            data-tag-slug={slug}
            data-tag-button="true"
        >
            <span>{format!("#{name}")}</span>
            {count
                .map(|total| {
                    view! {
                        <span class="rounded-full bg-slate-100 px-1.5 py-0.5 text-[10px] font-semibold text-slate-600">{total}</span>
                    }
                })}
        </button>
    }
}
