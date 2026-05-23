use crate::socket::{ServerEvent, SessionInfo, SocketSession};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
    scroll: u16,
    busy: bool,
    status: String,
    approval: Option<ApprovalState>,
    assistant_buf: String,
}

pub fn should_use_tui(plain: bool) -> bool {
    !plain && std::io::stdout().is_terminal() && std::io::stdin().is_terminal()
}

pub fn run(socket_path: &str) -> Result<(), String> {
    let (mut session, info) = SocketSession::connect(socket_path)?;
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, &mut session, info);
    ratatui::restore();
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
            .draw(|f| draw_ui(f, &app))
            .map_err(|e| e.to_string())?;

        let timeout = tick.saturating_sub(last_tick.elapsed());
        if event::poll(timeout).map_err(|e| e.to_string())? {
            if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
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
            scroll: 0,
            busy: false,
            status: format!("{} · {}", info.profile, info.model),
            approval: None,
            assistant_buf: String::new(),
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
        self.scroll = u16::MAX;
    }

    fn push_system(&mut self, text: impl Into<String>) {
        self.lines.push(ChatLine::System(text.into()));
        self.scroll = u16::MAX;
    }

    fn push_error(&mut self, text: impl Into<String>) {
        self.lines.push(ChatLine::Error(text.into()));
        self.scroll = u16::MAX;
    }

    fn flush_assistant(&mut self) {
        if !self.assistant_buf.is_empty() {
            let text = std::mem::take(&mut self.assistant_buf);
            self.lines.push(ChatLine::Assistant(text));
            self.scroll = u16::MAX;
        }
    }
}

fn handle_server_event(
    app: &mut App,
    session: &mut SocketSession,
    ev: ServerEvent,
) -> Result<(), String> {
    match ev {
        ServerEvent::Token(content) => {
            app.assistant_buf.push_str(&content);
        }
        ServerEvent::ToolCall { name, status } => {
            app.flush_assistant();
            app.lines.push(ChatLine::Tool {
                name: name.clone(),
                detail: tool_status_label(&status),
                ok: true,
            });
            app.status = format!("tool: {} …", name);
        }
        ServerEvent::ToolResult {
            name,
            success,
            output,
        } => {
            let preview: String = output.lines().take(3).collect::<Vec<_>>().join("\n");
            app.lines.push(ChatLine::Tool {
                name,
                detail: preview,
                ok: success,
            });
            app.scroll = u16::MAX;
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
            if !app.assistant_buf.is_empty() {
                app.flush_assistant();
            } else if !content.is_empty() && content != "(cancelled)" {
                app.lines.push(ChatLine::Assistant(content));
            }
            app.busy = false;
            app.status = format!("{} · {}", app.profile, app.model);
            app.scroll = u16::MAX;
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
            app.flush_assistant();
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
            app.scroll = app.scroll.saturating_sub(5);
        }
        KeyCode::PageDown => {
            app.scroll = app.scroll.saturating_add(5);
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
    session.send_chat(line)?;
    Ok(())
}

fn handle_slash(app: &mut App, session: &mut SocketSession, cmd: &str) -> Result<(), String> {
    let mut parts = cmd.split_whitespace();
    let head = parts.next().unwrap_or("").to_lowercase();
    match head.as_str() {
        "help" | "h" | "?" => {
            app.push_system(
                "Commands: /help /quit /clear /new /cancel /model /status [target]\n\
                 Approval: y approve · n dismiss",
            );
        }
        "quit" | "exit" | "q" => return Err("quit".into()),
        "clear" => {
            app.lines.clear();
            app.push_system("Cleared.");
        }
        "new" => {
            app.lines.clear();
            app.assistant_buf.clear();
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
        other => app.push_error(format!("Unknown command: /{}", other)),
    }
    Ok(())
}

fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled(" agnt ", Style::new().bold()),
            Span::raw(&app.status),
        ]),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" AgntOS "),
    );
    f.render_widget(header, chunks[0]);

    let chat_lines: Vec<Line> = app
        .lines
        .iter()
        .flat_map(line_to_ratatui)
        .collect();
    let chat_height = chunks[1].height.saturating_sub(2) as usize;
    let total = chat_lines.len();
    let max_scroll = total.saturating_sub(chat_height);
    let scroll = if app.scroll == u16::MAX {
        max_scroll
    } else {
        (app.scroll as usize).min(max_scroll)
    };

    let chat = Paragraph::new(Text::from(chat_lines))
        .block(Block::default().borders(Borders::ALL).title(" Chat "))
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
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

fn line_to_ratatui(line: &ChatLine) -> Vec<Line<'static>> {
    match line {
        ChatLine::User(s) => vec![Line::from(vec![
            Span::styled("you ", Style::new().cyan().bold()),
            Span::raw(s.clone()),
        ])],
        ChatLine::Assistant(s) => vec![Line::from(vec![
            Span::styled("agnt ", Style::new().green().bold()),
            Span::raw(s.clone()),
        ])],
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
                Line::from(Span::styled(
                    detail.clone(),
                    Style::new().add_modifier(Modifier::DIM),
                )),
            ]
        }
        ChatLine::System(s) => vec![Line::from(Span::styled(
            s.clone(),
            Style::new().add_modifier(Modifier::DIM),
        ))],
        ChatLine::Error(s) => vec![Line::from(Span::styled(
            s.clone(),
            Style::new().fg(Color::Red),
        ))],
    }
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
}
