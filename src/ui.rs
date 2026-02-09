use crate::app::AppState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, state: &mut AppState) {
    // Top-level vertical layout: title, main, bottom
    let main_layout = Layout::vertical([
        Constraint::Length(1), // Title bar
        Constraint::Min(0),    // Main content
        Constraint::Length(1), // Bottom bar
    ])
    .split(frame.area());

    // Title
    let title = Paragraph::new("Lazy-Dispatchrr")
        .style(Color::LightRed)
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(title, main_layout[0]);

    // Main area: left 25% (narrow) and right 75% (output)
    let areas = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(main_layout[1]);

    // Inside left area, split into three vertical columns (Repos | Branches | Workflows/Inputs)
    let left_columns = Layout::vertical([
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(34),
    ])
    .split(areas[0]);

    // 1) Repos list (left-most)
    let repo_items: Vec<ListItem> = state
        .ui.filtered_repo_indices
        .iter()
        .filter_map(|&i| state.data.repos.get(i))
        .map(|r| ListItem::new(r.name.clone()))
        .collect();
    let repos_highlight = if matches!(state.ui.focus, crate::app::Focus::Repo) {
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    let repos_border = if matches!(state.ui.focus, crate::app::Focus::Repo) {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    let repos_title = if state.ui.search_active && matches!(state.ui.focus, crate::app::Focus::Repo) {
        format!("Repos /{}█", state.ui.search_query)
    } else if state.ui.filtered_repo_indices.len() < state.data.repos.len() {
        format!("Repos [{}/{}]", state.ui.filtered_repo_indices.len(), state.data.repos.len())
    } else {
        "Repos".to_string()
    };
    let repos_list = List::new(repo_items)
        .block(Block::default().title(repos_title).borders(Borders::ALL).border_style(repos_border))
        .highlight_symbol(">> ")
        .highlight_style(repos_highlight);
    frame.render_stateful_widget(repos_list, left_columns[0], &mut state.ui.repos_state);

    // 2) Branches list (middle)
    let branch_items: Vec<ListItem> = state
        .ui.filtered_branch_indices
        .iter()
        .filter_map(|&i| state.data.branches.get(i))
        .map(|b| ListItem::new(b.clone()))
        .collect();
    let branches_highlight = if matches!(state.ui.focus, crate::app::Focus::Branches) {
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    let branches_border = if matches!(state.ui.focus, crate::app::Focus::Branches) {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    let branches_title = if state.ui.search_active && matches!(state.ui.focus, crate::app::Focus::Branches) {
        format!("Branches /{}█", state.ui.search_query)
    } else if state.ui.filtered_branch_indices.len() < state.data.branches.len() {
        format!("Branches [{}/{}]", state.ui.filtered_branch_indices.len(), state.data.branches.len())
    } else {
        "Branches".to_string()
    };
    let branches_list = List::new(branch_items)
        .block(Block::default().title(branches_title).borders(Borders::ALL).border_style(branches_border))
        .highlight_symbol(">> ")
        .highlight_style(branches_highlight);
    frame.render_stateful_widget(branches_list, left_columns[1], &mut state.ui.branches_state);

    // 3) Workflows / Inputs (right-most small column)
    let workflow_items: Vec<ListItem> = state
        .ui.filtered_workflow_indices
        .iter()
        .filter_map(|&i| state.data.workflows.get(i))
        .map(|w| ListItem::new(w.name.clone()))
        .collect();
    let workflows_highlight = if matches!(state.ui.focus, crate::app::Focus::Workflows) {
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    let workflows_border = if matches!(state.ui.focus, crate::app::Focus::Workflows) {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    let workflows_title = if state.ui.search_active && matches!(state.ui.focus, crate::app::Focus::Workflows) {
        format!("Workflows /{}█", state.ui.search_query)
    } else if state.ui.filtered_workflow_indices.len() < state.data.workflows.len() {
        format!("Workflows [{}/{}]", state.ui.filtered_workflow_indices.len(), state.data.workflows.len())
    } else {
        "Workflows".to_string()
    };
    let workflows_list = List::new(workflow_items)
        .block(Block::default().title(workflows_title).borders(Borders::ALL).border_style(workflows_border))
        .highlight_symbol(">> ")
        .highlight_style(workflows_highlight);
    frame.render_stateful_widget(workflows_list, left_columns[2], &mut state.ui.workflows_state);

    // Right area: big output panel (75% width)
    let output_border = if matches!(state.ui.focus, crate::app::Focus::Output) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let use_styled = state.ui.output_is_success && !state.ui.dispatch_output_lines.is_empty();
    if use_styled {
        let lines: Vec<Line> = state
            .ui.dispatch_output_lines
            .iter()
            .map(|(text, color)| {
                let fg = match color {
                    crate::app::DispatchOutputColor::Green => Color::Green,
                    crate::app::DispatchOutputColor::Yellow => Color::Yellow,
                    crate::app::DispatchOutputColor::White => Color::White,
                    crate::app::DispatchOutputColor::Blue => Color::LightBlue,
                };
                Line::from(Span::styled(text.clone(), Style::default().fg(fg)))
            })
            .collect();
        let output_paragraph = Paragraph::new(lines)
            .block(Block::default().title("Output").borders(Borders::ALL).border_style(output_border))
            .wrap(Wrap { trim: true });
        frame.render_widget(output_paragraph, areas[1]);
    } else {
        let output_text = state
            .ui.output
            .clone()
            .unwrap_or_else(|| "No output yet.".to_string());
        let output_style = if state.ui.output_is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        };
        let output_paragraph = Paragraph::new(output_text)
            .style(output_style)
            .block(Block::default().title("Output").borders(Borders::ALL).border_style(output_border))
            .wrap(Wrap { trim: true });
        frame.render_widget(output_paragraph, areas[1]);
    }

    // Bottom help bar
    let help_text = "Tab: focus | j/k: nav | /: search | r: replays | ?: help | q: quit";
    let help_paragraph = Paragraph::new(help_text).block(Block::default());
    frame.render_widget(help_paragraph, main_layout[2]);

    // Add Repo popup
    if state.ui.show_add_repo_popup {
        let area = frame.area();
        let popup_v = Layout::vertical([
            Constraint::Percentage(35),
            Constraint::Length(8),
            Constraint::Percentage(35),
        ])
        .split(area);

        let popup_h = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Min(40),
            Constraint::Percentage(25),
        ])
        .split(popup_v[1]);

        let popup_area = popup_h[1];
        frame.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .title(" Add Repo (Tab: switch field, Enter: submit, Esc: cancel) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightCyan));

        let inner = popup_block.inner(popup_area);
        frame.render_widget(popup_block, popup_area);

        let fields = Layout::vertical([
            Constraint::Length(1), // owner label + input
            Constraint::Length(1), // spacer
            Constraint::Length(1), // repo label + input
        ])
        .split(inner);

        let owner_style = if state.ui.add_repo_focus_owner {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let repo_style = if !state.ui.add_repo_focus_owner {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let cursor = "█";
        let owner_text = if state.ui.add_repo_focus_owner {
            format!("Owner: {}{}", state.ui.add_repo_owner, cursor)
        } else {
            format!("Owner: {}", state.ui.add_repo_owner)
        };
        let repo_text = if !state.ui.add_repo_focus_owner {
            format!("Repo:  {}{}", state.ui.add_repo_name, cursor)
        } else {
            format!("Repo:  {}", state.ui.add_repo_name)
        };

        frame.render_widget(Paragraph::new(owner_text).style(owner_style), fields[0]);
        frame.render_widget(Paragraph::new(repo_text).style(repo_style), fields[2]);
    }

    // Inputs popup — uses a scrollable Paragraph instead of per-row Layout
    // constraints, so that many inputs (even of the same type) never get
    // their rows collapsed to zero height by the layout solver.
    if state.ui.show_inputs_popup && !state.data.input_fields.is_empty() {
        let area = frame.area();
        // Use most of the screen height so the user can see many fields at once
        let popup_height = area.height.saturating_sub(6).max(10);

        let popup_v = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area);

        let popup_h = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Min(60),
            Constraint::Percentage(10),
        ])
        .split(popup_v[1]);

        let popup_area = popup_h[1];
        frame.render_widget(Clear, popup_area);

        let num_fields = state.data.input_fields.len();
        let title = format!(
            " Workflow Inputs [{}/{}] (j/k: navigate, Enter: edit, Tab: cycle choice, D: dispatch, S: save replay, Esc: cancel) ",
            state.ui.input_fields_selected + 1,
            num_fields,
        );
        let popup_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightMagenta));

        let inner = popup_block.inner(popup_area);
        frame.render_widget(popup_block, popup_area);

        // Build every field's lines into a single Vec<Line> for a scrollable Paragraph
        let cursor = "█";
        let mut lines: Vec<Line> = Vec::new();

        for (i, field) in state.data.input_fields.iter().enumerate() {
            let is_selected = i == state.ui.input_fields_selected;
            let is_editing = is_selected && state.ui.input_fields_editing;

            // Line 1: name + description
            let req_marker = if field.required { " *" } else { "" };
            let name_text = format!("{}{}: {}", field.name, req_marker, field.description);
            let name_style = if is_selected {
                Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(name_text, name_style)));

            // Line 2: type / default / options
            let mut meta_parts = vec![format!("  type: {}", field.input_type)];
            if !field.default_value.is_empty() {
                meta_parts.push(format!("default: {}", field.default_value));
            }
            if !field.options.is_empty() {
                meta_parts.push(format!("options: [{}]", field.options.join(", ")));
            }
            let meta_text = meta_parts.join(" | ");
            lines.push(Line::from(Span::styled(meta_text, Style::default().fg(Color::DarkGray))));

            // Line 3: value
            let val_display = if is_editing {
                format!("  > {}{}", field.value, cursor)
            } else if is_selected {
                format!("  > {}", field.value)
            } else {
                format!("    {}", field.value)
            };
            let val_style = if is_editing {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(Span::styled(val_display, val_style)));

            // Line 4: blank spacer between fields
            lines.push(Line::from(""));
        }

        // Scroll so the selected field is always visible.
        // Each field occupies 4 lines (name, meta, value, spacer).
        let lines_per_field: u16 = 4;
        let selected_top = state.ui.input_fields_selected as u16 * lines_per_field;
        let visible_height = inner.height;
        let scroll_offset = if selected_top + lines_per_field <= visible_height {
            0
        } else {
            // Keep selected field roughly in the top third of the view
            selected_top.saturating_sub(visible_height / 3)
        };

        let paragraph = Paragraph::new(lines).scroll((scroll_offset, 0));
        frame.render_widget(paragraph, inner);
    }

    // Replays popup
    if state.ui.show_replays_popup && !state.data.replays_list.is_empty() {
        let area = frame.area();
        let num_replays = state.data.replays_list.len();
        // Each replay: 2 lines (workflow + description) + 1 spacer
        let popup_height = ((num_replays as u16) * 3 + 4).min(area.height.saturating_sub(4));

        let popup_v = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area);

        let popup_h = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Min(50),
            Constraint::Percentage(15),
        ])
        .split(popup_v[1]);

        let popup_area = popup_h[1];
        frame.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .title(" Replays (j/k: navigate, Enter: run, d: delete, Esc: close) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightGreen));

        let inner = popup_block.inner(popup_area);
        frame.render_widget(popup_block, popup_area);

        let replay_items: Vec<ListItem> = state
            .data.replays_list
            .iter()
            .map(|r| {
                let text = format!("{}  ⟶  {}", r.workflow, r.description);
                ListItem::new(text)
            })
            .collect();

        let replay_list = List::new(replay_items)
            .highlight_symbol(">> ")
            .highlight_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(replay_list, inner, &mut state.ui.replays_state);
    }

    // Dispatch confirmation popup
    if state.ui.show_confirm_dispatch {
        let area = frame.area();
        let cmd_lines = state.ui.dispatch_command_preview.len() as u16 / area.width.saturating_sub(20) + 1;
        let popup_height = cmd_lines + 8;

        let popup_v = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area);

        let popup_h = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Min(50),
            Constraint::Percentage(15),
        ])
        .split(popup_v[1]);

        let popup_area = popup_h[1];
        frame.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .title(" Confirm Dispatch ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightYellow));

        let inner = popup_block.inner(popup_area);
        frame.render_widget(popup_block, popup_area);

        let confirm_text = format!(
            "Command to run:\n\n  {}\n\n(Y) to confirm  |  any other key to cancel",
            state.ui.dispatch_command_preview
        );
        let confirm_paragraph = Paragraph::new(confirm_text)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        frame.render_widget(confirm_paragraph, inner);
    }

    // Help popup
    if state.ui.show_help_popup {
        let area = frame.area();
        let popup_height = 24_u16.min(area.height.saturating_sub(4));

        let popup_v = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(popup_height),
            Constraint::Min(0),
        ])
        .split(area);

        let popup_h = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Min(50),
            Constraint::Percentage(20),
        ])
        .split(popup_v[1]);

        let popup_area = popup_h[1];
        frame.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .title(" Keybindings (? to close) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightCyan));

        let inner = popup_block.inner(popup_area);
        frame.render_widget(popup_block, popup_area);

        let help_lines: Vec<Line> = vec![
            Line::from(Span::styled("── General ──", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(vec![
                Span::styled("  Tab / Shift+Tab  ", Style::default().fg(Color::LightCyan)),
                Span::raw("Cycle focus between panels"),
            ]),
            Line::from(vec![
                Span::styled("  j/k  ↑/↓         ", Style::default().fg(Color::LightCyan)),
                Span::raw("Navigate lists"),
            ]),
            Line::from(vec![
                Span::styled("  Enter             ", Style::default().fg(Color::LightCyan)),
                Span::raw("Select / confirm action"),
            ]),
            Line::from(vec![
                Span::styled("  /                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Fuzzy search in focused panel"),
            ]),
            Line::from(vec![
                Span::styled("  a                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Add a new repo"),
            ]),
            Line::from(vec![
                Span::styled("  v                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Open repo in browser"),
            ]),
            Line::from(vec![
                Span::styled("  r                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Open saved replays"),
            ]),
            Line::from(vec![
                Span::styled("  i                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Edit workflow inputs"),
            ]),
            Line::from(vec![
                Span::styled("  q / Esc           ", Style::default().fg(Color::LightCyan)),
                Span::raw("Quit"),
            ]),
            Line::from(""),
            Line::from(Span::styled("── Inputs Popup ──", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(vec![
                Span::styled("  D                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Dispatch workflow"),
            ]),
            Line::from(vec![
                Span::styled("  S                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Save inputs as replay"),
            ]),
            Line::from(vec![
                Span::styled("  Tab / Shift+Tab   ", Style::default().fg(Color::LightCyan)),
                Span::raw("Cycle choice options"),
            ]),
            Line::from(""),
            Line::from(Span::styled("── Replays Popup ──", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(vec![
                Span::styled("  Enter             ", Style::default().fg(Color::LightCyan)),
                Span::raw("Run selected replay"),
            ]),
            Line::from(vec![
                Span::styled("  d                 ", Style::default().fg(Color::LightCyan)),
                Span::raw("Delete selected replay"),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_lines)
            .wrap(Wrap { trim: true });
        frame.render_widget(help_paragraph, inner);
    }
}