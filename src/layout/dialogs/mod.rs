use ratatui::layout::{Constraint, Layout, Rect};

use crate::layout::{Action, Job};

pub mod connections_dialog;
pub mod find_dialog;

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

fn build_dialog_job(action: &Action) -> Option<Job> {
    match action {
        Action::Submit => Some(Job::CloseDialog),
        Action::CloseDialog => Some(Job::CloseDialog),
        _ => None,
    }
}
