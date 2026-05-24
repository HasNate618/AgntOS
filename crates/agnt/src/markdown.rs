use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::borrow::Cow;

pub fn render_markdown(text: &str, body_style: Style) -> Vec<Line<'static>> {
    render_markdown_prefixed(text, None, body_style)
}

fn normalize_chat_newlines(text: &str) -> Cow<'_, str> {
    if !text.contains('\n') {
        return Cow::Borrowed(text);
    }
    Cow::Owned(
        text.replace("\r\n", "\n")
            .lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{line}  ")
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

pub fn render_markdown_prefixed(
    text: &str,
    prefix: Option<(&str, Style)>,
    body_style: Style,
) -> Vec<Line<'static>> {
    let text = normalize_chat_newlines(text);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut bold = false;
    let mut italic = false;
    let mut in_code_block = false;

    let mut push_line = |spans: &mut Vec<Span<'static>>| {
        if spans.is_empty() {
            return;
        }
        lines.push(Line::from(std::mem::take(spans)));
    };

    if let Some((p, style)) = prefix {
        current.push(Span::styled(p.to_string(), style));
    }

    let parser = Parser::new_ext(text.as_ref(), Options::ENABLE_STRIKETHROUGH);
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock { .. }) => {
                push_line(&mut current);
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                push_line(&mut current);
                in_code_block = false;
            }
            Event::Start(Tag::Strong) => bold = true,
            Event::End(TagEnd::Strong) => bold = false,
            Event::Start(Tag::Emphasis) => italic = true,
            Event::End(TagEnd::Emphasis) => italic = false,
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => push_line(&mut current),
            Event::End(TagEnd::Heading { .. }) => push_line(&mut current),
            Event::Start(Tag::Item) => current.push(Span::styled("• ", body_style)),
            Event::End(TagEnd::Item) => push_line(&mut current),
            Event::SoftBreak | Event::HardBreak => push_line(&mut current),
            Event::Start(Tag::Link { dest_url, .. }) => {
                current.push(Span::styled(
                    dest_url.to_string(),
                    body_style.fg(ratatui::style::Color::Cyan),
                ));
            }
            Event::End(TagEnd::Link) => {}
            Event::Text(t) => {
                let mut style = body_style;
                if in_code_block {
                    style = style.add_modifier(Modifier::DIM);
                } else {
                    if bold {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if italic {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                }
                current.push(Span::styled(t.to_string(), style));
            }
            Event::Code(t) => {
                current.push(Span::styled(
                    t.to_string(),
                    body_style.add_modifier(Modifier::DIM),
                ));
            }
            _ => {}
        }
    }
    push_line(&mut current);
    if lines.is_empty() {
        lines.push(Line::from(Span::raw("")));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_bold_and_code() {
        let lines = render_markdown("**hi** and `x`", Style::new());
        let flat: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.clone()))
            .collect();
        assert!(flat.contains("hi"));
        assert!(flat.contains("x"));
    }

    #[test]
    fn single_newlines_become_line_breaks() {
        let lines = render_markdown("line one\nline two", Style::new());
        assert!(lines.len() >= 2);
        assert!(lines[0].spans.iter().any(|s| s.content.contains("line one")));
        assert!(lines[1].spans.iter().any(|s| s.content.contains("line two")));
    }
}
