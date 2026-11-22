use std::{fs, path::Path};

use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Theme {
    pub style: Style,
    pub border: Style,
    pub border_selected: Style,
    pub item_selected: Style,
    pub item_selected_secondary: Style,
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub gray: Color,
    pub dark_gray: Color,
    pub light_red: Color,
    pub light_green: Color,
    pub light_yellow: Color,
    pub light_blue: Color,
    pub light_magenta: Color,
    pub light_cyan: Color,
    pub white: Color,
}

impl Theme {
    pub fn default_basic() -> Self {
        Theme {
            style: Style::new().bg(Color::Reset).fg(Color::Reset),
            border: Style::new().fg(Color::DarkGray),
            border_selected: Style::new().fg(Color::Blue),
            item_selected: Style::new().reversed(),
            item_selected_secondary: Style::new().bg(Color::DarkGray),
            black: Color::Black,
            red: Color::Red,
            green: Color::Green,
            yellow: Color::Yellow,
            blue: Color::Blue,
            magenta: Color::Magenta,
            cyan: Color::Cyan,
            gray: Color::Gray,
            dark_gray: Color::DarkGray,
            light_red: Color::LightRed,
            light_green: Color::LightGreen,
            light_yellow: Color::LightYellow,
            light_blue: Color::LightBlue,
            light_magenta: Color::LightMagenta,
            light_cyan: Color::LightCyan,
            white: Color::White,
        }
    }

    pub fn default_truecolor() -> Self {
        let bg = Color::Rgb(20, 20, 20);

        Theme {
            style: Style::new().bg(bg).fg(Color::White),
            border: Style::new().bg(bg).fg(Color::Rgb(64, 64, 64)),
            border_selected: Style::new().bg(bg).fg(Color::Rgb(21, 93, 252)),
            item_selected: Style::new().bg(Color::Rgb(85, 85, 85)).fg(Color::White),
            item_selected_secondary: Style::new().bg(Color::Rgb(38, 38, 38)).fg(Color::White),
            black: Color::Black,
            red: Color::Rgb(193, 0, 7),
            green: Color::Rgb(0, 130, 54),
            yellow: Color::Rgb(166, 95, 0),
            blue: Color::Rgb(20, 71, 230),
            magenta: Color::Rgb(67, 45, 215),
            cyan: Color::Rgb(0, 117, 149),
            gray: Color::Rgb(64, 64, 64),
            dark_gray: Color::Rgb(161, 161, 161),
            light_red: Color::Rgb(255, 100, 103),
            light_green: Color::Rgb(5, 223, 114),
            light_yellow: Color::Rgb(253, 199, 0),
            light_blue: Color::Rgb(81, 162, 255),
            light_magenta: Color::Rgb(124, 134, 255),
            light_cyan: Color::Rgb(0, 211, 242),
            white: Color::White,
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                return Err(error).with_context(|| format!("reading {}", path.display()));
            }
        };

        let theme: Theme =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;

        Ok(theme)
    }
}
