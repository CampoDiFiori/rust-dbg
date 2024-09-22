use std::io::Read;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use nix::sys::signal::Signal;
use ratatui::layout::{Rect, Size};
use ratatui::style::Style;
use ratatui::symbols::border::THICK;
use ratatui::{
    style::Stylize,
    widgets::{block::Title, Block, Paragraph, Widget},
};
use tracing::info;

use crate::files::ProjectFileCache;

pub fn run_tui(files: ProjectFileCache) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut tui = Tui { files };
    let file = tui
        .files
        .file("/home/pdudko/cool/examples/test.rs".as_ref())?;
    let mut file_content = String::new();
    let mut picker_content = String::new();

    file.read_to_string(&mut file_content)?;
    tui.files.to_buffer(&mut picker_content);

    let w = CoolWidget {
        file_content: &file_content,
        picker_content: &picker_content,
    };

    loop {
        terminal.draw(|frame| {
            frame.render_widget(&w, frame.area());
        })?;

        match event::read()? {
            Event::Key(key)
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') =>
            {
                break;
            }
            _ => {}
        }
    }
    ratatui::restore();

    Ok(())
}

pub struct Tui {
    files: ProjectFileCache,
}

struct CoolWidget<'a> {
    file_content: &'a str,
    picker_content: &'a str,
}

const PICKER_WIDTH: u16 = 30;

impl Widget for &CoolWidget<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let border_style = Style::new().blue();

        let title = Title::from(" Picker ".bold().style(border_style));
        let block = Block::bordered()
            .title(title.alignment(ratatui::layout::Alignment::Center))
            .border_set(THICK)
            .border_style(border_style);

        let picker_area = Rect::new(0, 0, PICKER_WIDTH, area.height);
        Paragraph::new(self.picker_content)
            .block(block)
            .render(picker_area, buf);

        let title = Title::from(" File ".bold().style(border_style));
        let block = Block::bordered()
            .title(title.alignment(ratatui::layout::Alignment::Center))
            .border_set(THICK)
            .border_style(border_style);

        let file_area = Rect::new(PICKER_WIDTH, 0, area.width - PICKER_WIDTH, area.height);
        Paragraph::new(self.file_content)
            .block(block)
            .render(file_area, buf);
    }
}
