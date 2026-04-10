use pulldown_cmark::{html, Options, Parser};

pub fn render_markdown(markdown: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
