use leptos::{component, view, IntoView};

use crate::components::markdown_editor::MarkdownEditor;

#[component]
pub fn PostEditorPage(post_id: Option<i64>) -> impl IntoView {
    let post_id_attr = post_id.map(|id| id.to_string()).unwrap_or_default();
    let is_edit = post_id.is_some();
    let heading = if is_edit { "Edit Post" } else { "Create Post" };
    let subheading = if is_edit {
        "Update title, content, tags, and publish state."
    } else {
        "Write a new post, manage tags, then save as draft or publish."
    };

    let script = r#"
(() => {
  const root = document.getElementById('post-editor-root');
  const form = document.getElementById('post-editor-form');
  const status = document.getElementById('post-editor-status');
  const saveBtn = document.getElementById('post-save');
  const titleInput = document.getElementById('post-title');
  const slugInput = document.getElementById('post-slug');
  const markdownInput = document.getElementById('post-markdown');
  const publishedInput = document.getElementById('post-published');
  const tagInput = document.getElementById('post-tag-input');
  const tagAdd = document.getElementById('post-tag-add');
  const tagList = document.getElementById('post-tags');
  const tagHidden = document.getElementById('post-tags-hidden');

  if (!root || !form || !status || !saveBtn || !titleInput || !slugInput || !markdownInput || !publishedInput || !tagInput || !tagAdd || !tagList || !tagHidden) {
    return;
  }

  const postId = Number(root.getAttribute('data-post-id') || '0') || null;
  let tags = [];

  const slugify = (value) => value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-');

  const syncTags = () => {
    tagHidden.value = JSON.stringify(tags);
    tagList.innerHTML = '';

    if (!tags.length) {
      const empty = document.createElement('p');
      empty.className = 'text-xs text-slate-500';
      empty.textContent = 'No tags selected.';
      tagList.appendChild(empty);
      return;
    }

    tags.forEach((tag) => {
      const item = document.createElement('button');
      item.type = 'button';
      item.className = 'inline-flex items-center gap-2 rounded-full bg-emerald-100 px-3 py-1 text-xs font-medium text-emerald-800';
      item.textContent = `#${tag}`;
      item.addEventListener('click', () => {
        tags = tags.filter((current) => current !== tag);
        syncTags();
      });
      tagList.appendChild(item);
    });
  };

  const setStatus = (message, isError = false) => {
    status.textContent = message;
    status.className = isError ? 'text-sm font-medium text-rose-600' : 'text-sm font-medium text-emerald-700';
  };

  titleInput.addEventListener('input', () => {
    if (!slugInput.value.trim()) {
      slugInput.value = slugify(titleInput.value);
    }
  });

  tagAdd.addEventListener('click', () => {
    const nextTag = slugify(tagInput.value);
    if (!nextTag) {
      return;
    }
    if (!tags.includes(nextTag)) {
      tags.push(nextTag);
      syncTags();
    }
    tagInput.value = '';
  });

  const loadPost = async () => {
    if (!postId) {
      syncTags();
      return;
    }

    try {
      const response = await fetch(`/server/posts/${postId}`);
      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || 'Unable to load post');
      }

      titleInput.value = data.title ?? '';
      slugInput.value = data.slug ?? '';
      markdownInput.value = data.markdown ?? '';
      publishedInput.checked = !!data.is_published;

      syncTags();
      setStatus('Post loaded.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to load post.', true);
    }
  };

  form.addEventListener('submit', async (event) => {
    event.preventDefault();

    const title = String(titleInput.value || '').trim();
    const slug = String(slugInput.value || '').trim();
    const markdown = String(markdownInput.value || '').trim();
    const isPublished = !!publishedInput.checked;

    if (!title || !slug || !markdown) {
      setStatus('Title, slug, and markdown content are required.', true);
      return;
    }

    saveBtn.setAttribute('disabled', 'true');
    setStatus('Saving...');

    const payload = {
      title,
      slug,
      markdown,
      is_published: isPublished,
      tags
    };

    const url = postId ? `/server/posts/${postId}` : '/server/posts/';
    const method = postId ? 'PUT' : 'POST';

    try {
      const response = await fetch(url, {
        method,
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload)
      });

      const data = await response.json().catch(() => ({}));

      if (!response.ok) {
        throw new Error(data.error || 'Failed to save post.');
      }

      setStatus('Post saved successfully. Redirecting...');
      window.setTimeout(() => window.location.assign('/admin'), 400);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to save post.', true);
      saveBtn.removeAttribute('disabled');
    }
  });

  loadPost();
})();
"#;

    view! {
        <div id="post-editor-root" data-post-id={post_id_attr} class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold tracking-tight text-slate-900 sm:text-3xl">{heading}</h1>
                <p class="mt-1 text-sm text-slate-600">{subheading}</p>
            </div>

            <form id="post-editor-form" class="space-y-5">
                <div class="grid gap-4 md:grid-cols-2">
                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-slate-700" for="post-title">"Title"</label>
                        <input
                            id="post-title"
                            name="title"
                            type="text"
                            required=true
                            class="w-full rounded-md border border-slate-300 px-3 py-2 text-slate-900 outline-none ring-emerald-200 transition focus:border-emerald-500 focus:ring"
                            placeholder="Post title"
                        />
                    </div>

                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-slate-700" for="post-slug">"Slug"</label>
                        <input
                            id="post-slug"
                            name="slug"
                            type="text"
                            required=true
                            class="w-full rounded-md border border-slate-300 px-3 py-2 text-slate-900 outline-none ring-emerald-200 transition focus:border-emerald-500 focus:ring"
                            placeholder="post-slug"
                        />
                    </div>
                </div>

                <MarkdownEditor />

                <div class="space-y-3 rounded-lg border border-slate-200 bg-white p-4">
                    <div>
                        <label class="mb-1 block text-sm font-medium text-slate-700" for="post-tag-input">"Tags"</label>
                        <div class="flex gap-2">
                            <input
                                id="post-tag-input"
                                type="text"
                                class="w-full rounded-md border border-slate-300 px-3 py-2 text-slate-900 outline-none ring-emerald-200 transition focus:border-emerald-500 focus:ring"
                                placeholder="Add tag and click Add"
                            />
                            <button
                                id="post-tag-add"
                                type="button"
                                class="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium text-slate-800 transition hover:bg-slate-100"
                            >
                                "Add"
                            </button>
                        </div>
                    </div>

                    <div id="post-tags" class="flex flex-wrap gap-2"></div>
                    <input id="post-tags-hidden" type="hidden" name="tags" value="[]" />
                </div>

                <label class="inline-flex items-center gap-2 text-sm font-medium text-slate-700">
                    <input id="post-published" type="checkbox" class="h-4 w-4 rounded border-slate-300 text-emerald-600 focus:ring-emerald-500" />
                    "Publish now"
                </label>

                <div class="flex items-center gap-3">
                    <button
                        id="post-save"
                        type="submit"
                        class="inline-flex items-center justify-center rounded-md bg-emerald-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-emerald-700 disabled:cursor-not-allowed disabled:bg-emerald-400"
                    >
                        "Save Post"
                    </button>
                    <a href="/admin" class="text-sm font-medium text-slate-600 hover:text-slate-900">"Cancel"</a>
                </div>

                <p id="post-editor-status" class="text-sm font-medium text-slate-600"></p>
            </form>
            <script inner_html=script></script>
        </div>
    }
}
