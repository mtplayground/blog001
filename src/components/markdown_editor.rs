use leptos::{component, view, IntoView};

#[component]
pub fn MarkdownEditor() -> impl IntoView {
    view! {
        <div class="space-y-2">
            <label class="block text-sm font-medium text-slate-700" for="post-markdown">"Content (Markdown)"</label>
            <textarea
                id="post-markdown"
                name="markdown"
                rows="18"
                class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 font-mono text-sm text-slate-900 outline-none ring-emerald-200 transition focus:border-emerald-500 focus:ring"
                placeholder="# Start writing your post in Markdown..."
                required=true
            ></textarea>
            <p class="text-xs text-slate-500">"Markdown is stored as raw text and rendered on public pages."</p>
        </div>
    }
}
