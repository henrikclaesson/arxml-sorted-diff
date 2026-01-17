use std::collections::HashSet;
use std::io::{self};
use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders, List, ListItem, Paragraph}, layout::{Constraint, Direction, Layout}, style::{Style, Color, Modifier}};
use crate::diff::{DiffNode, NodeStatus};

#[derive(Clone)]
struct VisibleRow {
    path: Vec<usize>,
    indent: usize,
    label: String,
    status: NodeStatus,
    has_children: bool,
}

fn build_rows(root: &DiffNode, expanded: &HashSet<String>) -> Vec<VisibleRow> {
    let mut rows = Vec::new();
    fn rec(node: &DiffNode, path: &mut Vec<usize>, indent: usize, expanded: &HashSet<String>, rows: &mut Vec<VisibleRow>) {
        let key = node.key.as_deref().unwrap_or(&node.tag).to_string();
        let path_str = path.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(".");
        let has_children = !node.children.is_empty();
        rows.push(VisibleRow { path: path.clone(), indent, label: key.clone(), status: node.status, has_children });

        if has_children && expanded.contains(&path_str) {
            for (i, child) in node.children.iter().enumerate() {
                path.push(i);
                rec(child, path, indent + 1, expanded, rows);
                path.pop();
            }
        }
    }

    let mut p = vec![0usize];
    rec(root, &mut p, 0, expanded, &mut rows);
    rows
}

fn path_to_string(path: &[usize]) -> String {
    path.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(".")
}

pub fn run_tui(root: &DiffNode) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut expanded: HashSet<String> = HashSet::new();
    // start with root expanded
    expanded.insert("0".to_string());

    let mut rows = build_rows(root, &expanded);
    let mut idx: usize = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            let items: Vec<ListItem> = rows.iter().enumerate().map(|(i, r)| {
                let mut txt = format!("{}{}", "  ".repeat(r.indent), r.label);
                if r.has_children {
                    let marker = if expanded.contains(&path_to_string(&r.path)) { "▾ " } else { "▸ " };
                    txt = format!("{}{}", "  ".repeat(r.indent), marker) + &r.label;
                }
                let style = match r.status {
                    NodeStatus::Added => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    NodeStatus::Removed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    NodeStatus::Changed => Style::default().fg(Color::Yellow),
                    NodeStatus::Unchanged => Style::default(),
                };
                let mut li = ListItem::new(txt).style(style);
                if i == idx { li = li.style(style.patch(Style::default().bg(Color::Blue))); }
                li
            }).collect();

            let list = List::new(items).block(Block::default().borders(Borders::ALL).title("ARXML Diff"));
            f.render_widget(list, chunks[0]);

            let help = Paragraph::new("j/k: move  Enter: expand/collapse  q: quit");
            f.render_widget(help, chunks[1]);
        })?;

        // handle input
        if event::poll(std::time::Duration::from_millis(200))? {
            if let CEvent::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Char('j') => { if idx + 1 < rows.len() { idx += 1; } }
                    KeyCode::Up | KeyCode::Char('k') => { if idx > 0 { idx -= 1; } }
                    KeyCode::Enter => {
                        // toggle expand on selected row if it has children
                        if let Some(sel) = rows.get(idx) {
                            if sel.has_children {
                                let path = path_to_string(&sel.path);
                                if expanded.contains(&path) { expanded.remove(&path); }
                                else { expanded.insert(path); }
                                rows = build_rows(root, &expanded);
                                if idx >= rows.len() { idx = rows.len().saturating_sub(1); }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
