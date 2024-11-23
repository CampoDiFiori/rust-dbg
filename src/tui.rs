use std::io::Read;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::symbols::border::THICK;
use ratatui::{
    style::Stylize,
    widgets::{block::Title, Block, Paragraph, Widget},
};

use crate::source_files::SourceFiles;

pub fn run_tui(files: SourceFiles) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut tui = Tui { files };
    let file = tui
        .files
        .file("/home/pdudko/cool/examples/test.rs".as_ref())?;
    let mut file_content = String::new();
    let mut picker_content = String::new();

    let breakpoint_lines = vec![1, 2, 3, 4, 5];

    file.read_to_string(&mut file_content)?;
    tui.files.to_buffer(&mut picker_content);

    let w = CoolWidget {
        file_content: &file_content,
        picker_content: &picker_content,
        focus: Focus::Picker,
        breakpoint_lines: &breakpoint_lines,
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
    files: SourceFiles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Picker,
    File,
}

struct CoolWidget<'a> {
    file_content: &'a str,
    picker_content: &'a str,
    focus: Focus,
    breakpoint_lines: &'a [u16],
}

const PICKER_WIDTH: u16 = 30;

impl Widget for &CoolWidget<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let style = |area: Focus| {
            if area == self.focus {
                Style::new().green()
            } else {
                Style::new().blue()
            }
        };

        // File picker
        {
            let border_style = style(Focus::Picker);
            let title = Title::from(" Picker ".bold().style(border_style));
            let block = Block::bordered()
                .title(title.alignment(ratatui::layout::Alignment::Center))
                .border_set(THICK)
                .border_style(border_style);

            let picker_area = Rect::new(0, 0, PICKER_WIDTH, area.height);
            Paragraph::new(self.picker_content)
                .block(block)
                .render(picker_area, buf);
        }

        // File content
        {
            let border_style = style(Focus::File);
            let title = Title::from(" File ".bold().style(border_style));
            let block = Block::bordered()
                .title(title.alignment(ratatui::layout::Alignment::Center))
                .border_set(THICK)
                .border_style(border_style);

            // A column with breakpoint dots
            let mut breakpoints_content = String::new();
            let breakpoints_rect = Rect::new(PICKER_WIDTH, 0, 2, area.height);
            for i in 0..area.height {
                if self.breakpoint_lines.contains(&i) {
                    breakpoints_content.push_str("B");
                } else {
                    breakpoints_content.push_str(" ");
                }
            }
            Paragraph::new(breakpoints_content).render(breakpoints_rect, buf);

            let file_area = Rect::new(
                PICKER_WIDTH + 2,
                0,
                area.width - PICKER_WIDTH - 2,
                area.height,
            );
            Paragraph::new(self.file_content)
                .block(block)
                .render(file_area, buf);
        }
    }
}
