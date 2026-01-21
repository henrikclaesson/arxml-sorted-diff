use std::collections::HashSet;
use std::io::{self};
use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders, List, ListItem, Paragraph, ListState}, layout::{Constraint, Direction, Layout}, style::{Style, Color, Modifier}};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use quick_xml::events::Event;
use std::io::Cursor;
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
    let mut state = ListState::default();
    state.select(Some(idx));

    #[derive(PartialEq)]
    enum ViewMode { Unified, SideBySide }
    let mut view_mode = ViewMode::Unified;
    let mut show_raw = false;

    // transient status message with expiry
    let mut status_msg: Option<(String, Instant)> = None;

    // helper to find a node by its path (path[0] == 0 is root)
    fn node_by_path<'a>(root: &'a DiffNode, path: &'a [usize]) -> &'a DiffNode {
        let mut node = root;
        for idx in path.iter().skip(1) {
            node = &node.children[*idx];
        }
        node
    }

    // pretty-print an XML fragment using quick-xml Writer (falls back to input on error)
    fn pretty_print_xml(xml: &str) -> String {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut out: Vec<u8> = Vec::new();
        let mut writer = Writer::new_with_indent(Cursor::new(&mut out), b' ', 4);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    if writer.write_event(Event::Start(e.into_owned())).is_err() { return xml.to_string(); }
                }
                Ok(Event::End(e)) => {
                    if writer.write_event(Event::End(e.into_owned())).is_err() { return xml.to_string(); }
                }
                Ok(Event::Text(t)) => {
                    if writer.write_event(Event::Text(t.into_owned())).is_err() { return xml.to_string(); }
                }
                Ok(Event::Empty(e)) => {
                    if writer.write_event(Event::Empty(e.into_owned())).is_err() { return xml.to_string(); }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => return xml.to_string(),
            }
            buf.clear();
        }
        match std::str::from_utf8(&out) {
            Ok(s) => s.to_string(),
            Err(_) => xml.to_string(),
        }
    }

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = if show_raw {
                Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                    .split(size)
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(1), Constraint::Length(6)].as_ref())
                    .split(size)
            };

            // Top area: either unified full-width or side-by-side two columns
            match view_mode {
                ViewMode::Unified => {
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
                    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("ARXML Diff (Unified)"));
                    f.render_stateful_widget(list, chunks[0], &mut state);
                }
                ViewMode::SideBySide => {
                    let top = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref()).split(chunks[0]);
                    let mut left_items: Vec<ListItem> = Vec::new();
                    let mut right_items: Vec<ListItem> = Vec::new();
                    for (i, r) in rows.iter().enumerate() {
                        let indent = "  ".repeat(r.indent);
                        let (ltext, rtext, lstyle, rstyle) = match r.status {
                            NodeStatus::Added => ("".to_string(), format!("{}{}", indent, r.label), Style::default(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            NodeStatus::Removed => (format!("{}{}", indent, r.label), "".to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD), Style::default()),
                            NodeStatus::Changed => (format!("{}{}", indent, r.label), format!("{}{}", indent, r.label), Style::default().fg(Color::Yellow), Style::default().fg(Color::Yellow)),
                            NodeStatus::Unchanged => (format!("{}{}", indent, r.label), format!("{}{}", indent, r.label), Style::default(), Style::default()),
                        };
                        let mut li_l = ListItem::new(ltext).style(lstyle);
                        let mut li_r = ListItem::new(rtext).style(rstyle);
                        if i == idx {
                            li_l = li_l.style(lstyle.patch(Style::default().bg(Color::Blue)));
                            li_r = li_r.style(rstyle.patch(Style::default().bg(Color::Blue)));
                        }
                        left_items.push(li_l);
                        right_items.push(li_r);
                    }
                    let left_list = List::new(left_items).block(Block::default().borders(Borders::ALL).title("Left"));
                    let right_list = List::new(right_items).block(Block::default().borders(Borders::ALL).title("Right"));
                    f.render_stateful_widget(left_list, top[0], &mut state);
                    f.render_stateful_widget(right_list, top[1], &mut state);
                }
            }

            // Bottom area: either raw XML view or help/status text
            if show_raw {
                if let Some(sel) = rows.get(idx) {
                    let node = node_by_path(root, &sel.path);
                    let mut raw = String::new();
                    raw.push_str("-- Left --\n");
                    if let Some(l) = &node.left_xml { raw.push_str(&pretty_print_xml(l)); } else { raw.push_str("<none>"); }
                    raw.push_str("\n\n-- Right --\n");
                    if let Some(r) = &node.right_xml { raw.push_str(&pretty_print_xml(r)); } else { raw.push_str("<none>"); }
                    let para = Paragraph::new(raw).block(Block::default().borders(Borders::ALL).title("Raw XML (r toggles, c: export)"))
                        .wrap(ratatui::widgets::Wrap { trim: true });
                    f.render_widget(para, chunks[1]);
                }
            } else {
                // help + (optional) status message
                let mut help_text = String::from("j/k: move  Enter: expand/collapse  v: toggle view  r: toggle raw  c: export  q: quit");
                if let Some((msg, when)) = &status_msg {
                    // expire after 3 seconds
                    if when.elapsed().as_secs() < 3 {
                        help_text.push_str("    ");
                        help_text.push_str(msg);
                    }
                }
                let help = Paragraph::new(help_text);
                f.render_widget(help, chunks[1]);
            }
        })?;

        // handle input
        if event::poll(std::time::Duration::from_millis(200))? {
            if let CEvent::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Char('j') => { if idx + 1 < rows.len() { idx += 1; state.select(Some(idx)); } }
                    KeyCode::Up | KeyCode::Char('k') => { if idx > 0 { idx -= 1; state.select(Some(idx)); } }
                    KeyCode::Char('v') => { view_mode = if view_mode == ViewMode::Unified { ViewMode::SideBySide } else { ViewMode::Unified }; }
                    KeyCode::Char('r') => { show_raw = !show_raw; }
                    KeyCode::Char('c') => {
                        // Export selected node (pretty-printed) to a file
                        if let Some(sel) = rows.get(idx) {
                            let node = node_by_path(root, &sel.path);
                            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                            let fname = format!("arxml-{}-{}.xml", node.tag, ts);
                            let left = node.left_xml.as_deref().map(pretty_print_xml);
                            let right = node.right_xml.as_deref().map(pretty_print_xml);
                            let mut out = String::new();
                            out.push_str("<!-- Exported by arxml-diff -->\n");
                            out.push_str("<!-- Left -->\n");
                            out.push_str(left.as_deref().unwrap_or("<none>"));
                            out.push_str("\n\n<!-- Right -->\n");
                            out.push_str(right.as_deref().unwrap_or("<none>"));
                            if std::fs::write(&fname, out).is_ok() {
                                status_msg = Some((format!("Exported to {}", fname), Instant::now()));
                            } else {
                                status_msg = Some((format!("Failed to write {}", fname), Instant::now()));
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // toggle expand on selected row if it has children
                        if let Some(sel) = rows.get(idx) {
                            if sel.has_children {
                                let path = path_to_string(&sel.path);
                                if expanded.contains(&path) { expanded.remove(&path); }
                                else { expanded.insert(path); }
                                rows = build_rows(root, &expanded);
                                if idx >= rows.len() { idx = rows.len().saturating_sub(1); }
                                state.select(Some(idx));
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
