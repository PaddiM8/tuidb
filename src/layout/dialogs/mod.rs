use ratatui::layout::{Constraint, Layout, Rect};

use crate::layout::{Action, CommandKind};

pub mod connections_dialog;
pub mod find_dialog;
pub mod tab_dialog;

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let [_, v, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(h),
        Constraint::Fill(1),
    ])
    .areas(area);

    let [_, c, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(w),
        Constraint::Fill(1),
    ])
    .areas(v);

    c
}

fn build_dialog_action(action: &CommandKind) -> Option<Action> {
    match action {
        CommandKind::Submit => Some(Action::CloseDialog),
        CommandKind::CloseDialog => Some(Action::CloseDialog),
        _ => None,
    }
}
