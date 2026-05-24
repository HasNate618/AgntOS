use crate::markdown;
use crate::skills;
use crate::socket::{ServerEvent, SessionInfo, SocketSession};
use agnt_common::wire::TokenChannel;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::io::IsTerminal;
use std::time::{Duration, Instant};

#[derive(Clone)]
enum ChatLine {
    User(String),
    Thinking(String),
    Assistant(String),
    Tool {
        name: String,
        detail: String,
        ok: bool,
    },
    System(String),
    Error(String),
}

struct ApprovalState {
    proposal_id: String,
    summary: String,
}

struct App {
    profile: String,
    model: String,
    lines: Vec<ChatLine>,
    input: String,
    scroll_y: usize,
    follow_tail: bool,
    last_viewport_rows: usize,
    busy: bool,
    status: String,
    approval: Option<ApprovalState>,
    assistant_buf: String,
    thinking_buf: String,
}

pub fn should_use_tui(plain: bool) -> bool {
    !plain && std::io::stdout().is_terminal() && std::io::stdin().is_terminal()
}

pub fn run(socket_path: &str) -> Result<(), String> {
    let (mut session, info) = SocketSession::connect(socket_path)?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::event::EnableMouseCapture)
        .map_err(|e| e.to_string())?;
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, &mut session, info);
    ratatui::restore();
    let _ = crossterm::execute!(stdout, crossterm::event::DisableMouseCapture);
    result
}

fn run_loop(
    terminal: &mut ratatui::DefaultTerminal,
    session: &mut SocketSession,
    info: SessionInfo,
) -> Result<(), String> {
    let mut app = App::new(info);
    let tick = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    loop {
        terminal
            .draw(|f| draw_ui(f, &mut app))
            .map_err(|e| e.to_string())?;

        let timeout = tick.saturating_sub(last_tick.elapsed());
        if event::poll(timeout).map_err(|e| e.to_string())? {
            match event::read().map_err(|e| e.to_string())? {
                Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if app.approval.is_some() {
                    if handle_approval_key(&mut app, session, key.code, key.modifiers)? {
                        continue;
                    }
                } else if handle_input_key(&mut app, session, key.code, key.modifiers)? {
                    return Ok(());
                }
                }
                Event::Mouse(m) => {
                    match m.kind {
                        MouseEventKind::ScrollUp => {
                            app.follow_tail = false;
                            app.scroll_y = app.scroll_y.saturating_sub(3);
                        }
                        MouseEventKind::ScrollDown => {
                            app.follow_tail = false;
                            app.scroll_y = app.scroll_y.saturating_add(3);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        while let Some(ev) = session.try_recv_event() {
            handle_server_event(&mut app, session, ev)?;
        }

        if last_tick.elapsed() >= tick {
            last_tick = Instant::now();
        }
    }
}

impl App {
    fn new(info: SessionInfo) -> Self {
        let mut app = Self {
            profile: info.profile.clone(),
            model: info.model.clone(),
            lines: Vec::new(),
            input: String::new(),
            scroll_y: 0,
            follow_tail: true,
            last_viewport_rows: 10,
            busy: false,
            status: format!("{} · {}", info.profile, info.model),
            approval: None,
            assistant_buf: String::new(),
            thinking_buf: String::new(),
        };
        app.push_system(format!(
            "Connected (model: {}). /help for commands.",
            info.model
        ));
        if !info.pending_proposals.is_empty() {
            app.push_system(format!(
                "Pending proposals: {}",
                info.pending_proposals.join(", ")
            ));
        }
        app
    }

    fn push_user(&mut self, text: impl Into<String>) {
        self.lines.push(ChatLine::User(text.into()));
        self.scroll_bottom();
    }

    fn push_system(&mut self, text: impl Into<String>) {
        self.lines.push(ChatLine::System(text.into()));
        self.scroll_bottom();
    }

    fn push_error(&mut self, text: impl Into<String>) {
        self.lines.push(ChatLine::Error(text.into()));
        self.scroll_bottom();
    }

    fn flush_thinking(&mut self) {
        if !self.thinking_buf.is_empty() {
            let text = std::mem::take(&mut self.thinking_buf);
            self.lines.push(ChatLine::Thinking(text));
            self.scroll_bottom();
        }
    }

    fn flush_streaming(&mut self) {
        self.flush_thinking();
        if !self.assistant_buf.is_empty() {
            let text = std::mem::take(&mut self.assistant_buf);
            self.lines.push(ChatLine::Assistant(text));
            self.scroll_bottom();
        }
    }

    fn scroll_bottom(&mut self) {
        if self.follow_tail {
            self.scroll_y = usize::MAX;
        }
    }

    fn page_step(&self) -> usize {
        self.last_viewport_rows.max(1)
    }
}

fn handle_server_event(
    app: &mut App,
    session: &mut SocketSession,
    ev: ServerEvent,
) -> Result<(), String> {
    match ev {
        ServerEvent::Token { content, channel } => match channel {
            TokenChannel::Thinking => {
                app.thinking_buf.push_str(&content);
                app.scroll_bottom();
            }
            TokenChannel::Content => {
                app.assistant_buf.push_str(&content);
                app.scroll_bottom();
            }
        },
        ServerEvent::ToolCall { name, status, args } => {
            app.flush_streaming();
            let mut detail = format_tool_args(&args);
            let status_label = tool_status_label(&status);
            if !detail.is_empty() {
                detail = format!("{}\n{}", status_label, detail);
            } else {
                detail = status_label;
            }
            app.lines.push(ChatLine::Tool {
                name: name.clone(),
                detail,
                ok: true,
            });
            app.status = format!("tool: {} …", name);
        }
        ServerEvent::ToolResult {
            name,
            success,
            output,
        } => {
            let preview: String = output.lines().take(8).collect::<Vec<_>>().join("\n");
            app.lines.push(ChatLine::Tool {
                name,
                detail: preview,
                ok: success,
            });
            app.follow_tail = true;
            app.scroll_y = usize::MAX;
        }
        ServerEvent::ApprovalRequest {
            proposal_id,
            summary,
        } => {
            app.approval = Some(ApprovalState {
                proposal_id,
                summary,
            });
            app.status = "approval required — y/n".into();
        }
        ServerEvent::TurnComplete { content } => {
            app.flush_streaming();
            if !content.is_empty() && content != "(cancelled)" {
                let already = matches!(
                    app.lines.last(),
                    Some(ChatLine::Assistant(c)) if c == &content
                );
                if !already {
                    app.lines.push(ChatLine::Assistant(content));
                }
            }
            app.busy = false;
            app.status = format!("{} · {}", app.profile, app.model);
            app.follow_tail = true;
            app.scroll_y = usize::MAX;
        }
        ServerEvent::Ready(info) => {
            app.profile = info.profile;
            app.model = info.model;
            app.status = format!("{} · {}", app.profile, app.model);
            app.push_system("Session refreshed.");
        }
        ServerEvent::StatusResponse { target, output } => {
            app.busy = false;
            app.push_system(format!("--- inspect {} ---\n{}", target, output.trim()));
            app.status = format!("{} · {}", app.profile, app.model);
        }
        ServerEvent::AuditResponse { lines } => {
            app.busy = false;
            if lines.is_empty() {
                app.push_system("No audit entries.");
            } else {
                app.push_system(format!("--- audit ({} entries) ---", lines.len()));
                for line in lines.iter().take(20) {
                    app.push_system(line.clone());
                }
            }
            app.status = format!("{} · {}", app.profile, app.model);
        }
        ServerEvent::Error { message } => {
            app.flush_streaming();
            app.push_error(message);
            app.busy = false;
            app.status = format!("{} · {}", app.profile, app.model);
        }
    }
    let _ = session;
    Ok(())
}

fn handle_approval_key(
    app: &mut App,
    session: &mut SocketSession,
    code: KeyCode,
    _mods: KeyModifiers,
) -> Result<bool, String> {
    let Some(approval) = app.approval.take() else {
        return Ok(false);
    };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            session.send_approve(&approval.proposal_id)?;
            app.push_system(format!("Approved: {}", approval.summary));
            app.status = "applying…".into();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            session.send_dismiss(&approval.proposal_id, "declined in TUI")?;
            app.push_system(format!("Dismissed: {}", approval.summary));
            app.status = format!("{} · {}", app.profile, app.model);
        }
        _ => {
            app.approval = Some(approval);
            return Ok(true);
        }
    }
    Ok(true)
}

fn handle_input_key(
    app: &mut App,
    session: &mut SocketSession,
    code: KeyCode,
    mods: KeyModifiers,
) -> Result<bool, String> {
    match code {
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => return Ok(true),
        KeyCode::Esc => {
            app.input.clear();
        }
        KeyCode::Enter => {
            if app.busy || app.approval.is_some() {
                return Ok(false);
            }
            let line = app.input.trim().to_string();
            app.input.clear();
            if line.is_empty() {
                return Ok(false);
            }
            match submit_line(app, session, &line) {
                Ok(()) => {}
                Err(e) if e == "quit" => return Ok(true),
                Err(e) => return Err(e),
            }
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        KeyCode::PageUp => {
            app.follow_tail = false;
            app.scroll_y = app.scroll_y.saturating_sub(app.page_step());
        }
        KeyCode::PageDown => {
            app.follow_tail = false;
            app.scroll_y = app.scroll_y.saturating_add(app.page_step());
        }
        KeyCode::Home => {
            app.follow_tail = false;
            app.scroll_y = 0;
        }
        KeyCode::End => {
            app.follow_tail = true;
            app.scroll_y = usize::MAX;
        }
        _ => {}
    }
    Ok(false)
}

fn submit_line(app: &mut App, session: &mut SocketSession, line: &str) -> Result<(), String> {
    if let Some(cmd) = line.strip_prefix('/') {
        return handle_slash(app, session, cmd);
    }
    app.push_user(line);
    app.busy = true;
    app.status = "thinking…".into();
    app.assistant_buf.clear();
    app.thinking_buf.clear();
    session.send_chat(line)?;
    Ok(())
}

fn handle_slash(app: &mut App, session: &mut SocketSession, cmd: &str) -> Result<(), String> {
    let mut parts = cmd.split_whitespace();
    let head = parts.next().unwrap_or("").to_lowercase();
    match head.as_str() {
        "help" | "h" | "?" => {
            let skill_list = skills::list_skills().join(", ");
            app.push_system(format!(
                "Commands: /help /quit /clear /new /cancel /model /status [target]\n\
                 /audit [search Q] · /audit show <id> · /skills\n\
                 Skills: {}\n\
                 Approval: y/n · Home/End scroll · PgUp/PgDn",
                if skill_list.is_empty() {
                    "(none installed)".into()
                } else {
                    skill_list
                }
            ));
        }
        "skills" => {
            let list = skills::list_skills();
            if list.is_empty() {
                app.push_system("No skills in /etc/agntos/skills or ~/.config/agntos/skills");
            } else {
                app.push_system(format!("Skills: {}", list.join(", ")));
            }
        }
        "audit" => {
            let sub = parts.next().unwrap_or("list");
            app.busy = true;
            match sub {
                "show" => {
                    let id = parts.next().ok_or_else(|| "usage: /audit show <id>".to_string())?;
                    app.status = "audit show…".into();
                    session.send_audit_show(id)?;
                }
                "search" => {
                    let q = parts.collect::<Vec<_>>().join(" ");
                    if q.is_empty() {
                        return Err("usage: /audit search <query>".into());
                    }
                    app.status = "audit search…".into();
                    session.send_audit_search(&q, 20)?;
                }
                _ => {
                    app.status = "audit list…".into();
                    session.send_audit_list(20)?;
                }
            }
        }
        "quit" | "exit" | "q" => return Err("quit".into()),
        "clear" => {
            app.lines.clear();
            app.push_system("Cleared.");
        }
        "new" => {
            app.lines.clear();
            app.assistant_buf.clear();
            app.thinking_buf.clear();
            let _ = session.send_init();
            app.push_system("New session.");
        }
        "cancel" => {
            session.send_cancel()?;
            app.busy = false;
            app.push_system("Cancel sent.");
        }
        "model" => {
            app.push_system(format!("Model: {} (profile: {})", app.model, app.profile));
        }
        "status" => {
            let target = parts.next().unwrap_or("system");
            app.busy = true;
            app.status = format!("inspect {}…", target);
            session.send_status(target)?;
        }
        other if skills::list_skills().iter().any(|s| s == other) => {
            let prompt = skills::skill_prompt(other)?;
            app.push_system(format!("Loaded skill /{}", other));
            app.busy = true;
            app.status = format!("skill {}…", other);
            app.assistant_buf.clear();
            app.thinking_buf.clear();
            session.send_chat(&prompt)?;
        }
        other => app.push_error(format!("Unknown command: /{}", other)),
    }
    Ok(())
}

fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let scroll_hint = if app.follow_tail {
        ""
    } else {
        " · scroll"
    };
    let header = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled(" agnt ", Style::new().bold()),
            Span::raw(format!("{}{}", app.status, scroll_hint)),
        ]),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" AgntOS "),
    );
    f.render_widget(header, chunks[0]);

    let chat_lines = build_chat_lines(app);
    let inner_width = chunks[1].width.saturating_sub(2);
    let chat_height = chunks[1].height.saturating_sub(2) as usize;
    app.last_viewport_rows = chat_height.max(1);

    let chat_block = Block::default().borders(Borders::ALL).title(" Chat ");
    let chat_text = Text::from(chat_lines);
    let total_rows = wrapped_line_count(&chat_text.lines, inner_width);
    let chat_para = Paragraph::new(chat_text)
        .block(chat_block)
        .wrap(Wrap { trim: false });
    let max_scroll = total_rows.saturating_sub(chat_height);
    let scroll_y = if app.follow_tail || app.scroll_y == usize::MAX {
        max_scroll
    } else {
        app.scroll_y.min(max_scroll)
    };
    app.scroll_y = scroll_y;

    let chat = chat_para.scroll((scroll_y as u16, 0));
    f.render_widget(chat, chunks[1]);

    let input_title = if app.busy {
        " Input (busy) "
    } else {
        " Input "
    };
    let input = Paragraph::new(format!("> {}", app.input))
        .block(Block::default().borders(Borders::ALL).title(input_title));
    f.render_widget(input, chunks[2]);

    if let Some(ref approval) = app.approval {
        draw_approval(f, approval);
    }
}

fn build_chat_lines(app: &App) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = app.lines.iter().flat_map(line_to_ratatui).collect();
    if !app.thinking_buf.is_empty() {
        out.extend(markdown::render_markdown_prefixed(
            &app.thinking_buf,
            Some(("think ", thinking_label_style())),
            thinking_body_style(),
        ));
    }
    if !app.assistant_buf.is_empty() {
        out.extend(markdown::render_markdown_prefixed(
            &app.assistant_buf,
            Some(("agnt ", assistant_label_style())),
            Style::new(),
        ));
    }
    out
}

fn thinking_label_style() -> Style {
    Style::new()
        .add_modifier(Modifier::DIM | Modifier::ITALIC | Modifier::BOLD)
}

fn thinking_body_style() -> Style {
    Style::new().add_modifier(Modifier::DIM | Modifier::ITALIC)
}

fn assistant_label_style() -> Style {
    Style::new().green().bold()
}

fn line_to_ratatui(line: &ChatLine) -> Vec<Line<'static>> {
    match line {
        ChatLine::User(s) => multiline_prefix_lines("you ", Style::new().cyan().bold(), s),
        ChatLine::Thinking(s) => markdown::render_markdown_prefixed(
            s,
            Some(("think ", thinking_label_style())),
            thinking_body_style(),
        ),
        ChatLine::Assistant(s) => markdown::render_markdown_prefixed(
            s,
            Some(("agnt ", assistant_label_style())),
            Style::new(),
        ),
        ChatLine::Tool { name, detail, ok } => {
            let style = if *ok {
                Style::new().yellow()
            } else {
                Style::new().red()
            };
            vec![
                Line::from(vec![
                    Span::styled(format!("tool {} ", name), style.bold()),
                    Span::styled(
                        if *ok { "ok" } else { "fail" },
                        style,
                    ),
                ]),
            ]
            .into_iter()
            .chain(multiline_plain_lines(
                detail,
                Style::new().add_modifier(Modifier::DIM),
            ))
            .collect()
        }
        ChatLine::System(s) => {
            multiline_plain_lines(s, Style::new().add_modifier(Modifier::DIM))
        }
        ChatLine::Error(s) => vec![Line::from(Span::styled(
            s.clone(),
            Style::new().fg(Color::Red),
        ))],
    }
}

fn wrapped_line_count(lines: &[Line], width: u16) -> usize {
    let w = usize::from(width.max(1));
    lines
        .iter()
        .map(|line| {
            let line_w = line.width();
            if line_w == 0 {
                1
            } else {
                line_w.div_ceil(w)
            }
        })
        .sum()
}

fn multiline_prefix_lines(label: &str, label_style: Style, text: &str) -> Vec<Line<'static>> {
    text.split('\n')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                Line::from(vec![
                    Span::styled(label.to_string(), label_style),
                    Span::raw(part.to_string()),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" ".repeat(label.len())),
                    Span::raw(part.to_string()),
                ])
            }
        })
        .collect()
}

fn multiline_plain_lines(text: &str, style: Style) -> Vec<Line<'static>> {
    text.split('\n')
        .map(|part| Line::from(Span::styled(part.to_string(), style)))
        .collect()
}

fn draw_approval(f: &mut Frame, approval: &ApprovalState) {
    let area = centered_rect(70, 40, f.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Apply change? ")
        .style(Style::new().bg(Color::Black));
    let text = vec![
        Line::from(Span::styled(
            approval.summary.clone(),
            Style::new().bold(),
        )),
        Line::from(""),
        Line::from(format!("Proposal: {}", approval.proposal_id)),
        Line::from(""),
        Line::from(vec![
            Span::styled(" y ", Style::new().green().bold()),
            Span::raw("approve  "),
            Span::styled(" n ", Style::new().red().bold()),
            Span::raw("dismiss"),
        ]),
    ];
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn format_tool_args(args: &serde_json::Value) -> String {
    if args.is_null() || args.as_object().is_some_and(|o| o.is_empty()) {
        return String::new();
    }
    let pretty = serde_json::to_string_pretty(args).unwrap_or_else(|_| args.to_string());
    const MAX: usize = 480;
    if pretty.len() <= MAX {
        pretty
    } else {
        let end = pretty.floor_char_boundary(MAX);
        format!("{}…", &pretty[..end])
    }
}

fn tool_status_label(status: &agnt_common::wire::ToolCallStatus) -> String {
    use agnt_common::wire::ToolCallStatus;
    match status {
        ToolCallStatus::Running => "running".into(),
        ToolCallStatus::Done => "done".into(),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_tui_when_tty() {
        assert!(!should_use_tui(true));
    }

    #[test]
    fn wrapped_line_count_splits_long_lines() {
        let lines = vec![Line::from("x".repeat(80))];
        assert_eq!(wrapped_line_count(&lines, 20), 4);
    }
}
