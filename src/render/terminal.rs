use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout, Write};
pub struct Terminal {
    out: Stdout,
    previous: Vec<u8>,
    width: usize,
}
impl Terminal {
    pub fn enter() -> io::Result<Self> {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            previous_hook(info);
        }));
        terminal::enable_raw_mode()?;
        let mut terminal = Self {
            out: io::stdout(),
            previous: Vec::new(),
            width: 0,
        };
        execute!(
            terminal.out,
            EnterAlternateScreen,
            Hide,
            EnableMouseCapture,
            SetForegroundColor(Color::White),
            SetBackgroundColor(Color::Black)
        )?;
        Ok(terminal)
    }
    pub fn present(&mut self, cells: &[u8], width: usize) -> io::Result<()> {
        if width == 0 {
            return Ok(());
        }
        if self.previous.len() != cells.len() || self.width != width {
            self.previous = vec![0; cells.len()];
            self.width = width;
        }
        for (y, row) in cells.chunks(width).enumerate() {
            let old = &self.previous[y * width..y * width + row.len()];
            let Some(first) = row.iter().zip(old).position(|(a, b)| a != b) else {
                continue;
            };
            let last = row.iter().zip(old).rposition(|(a, b)| a != b).unwrap();
            queue!(
                self.out,
                MoveTo(first as u16, y as u16),
                Print(std::str::from_utf8(&row[first..=last]).unwrap())
            )?;
        }
        self.out.flush()?;
        self.previous.copy_from_slice(cells);
        Ok(())
    }
}
fn restore() {
    let _ = execute!(
        io::stdout(),
        ResetColor,
        DisableMouseCapture,
        Show,
        LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
}
impl Drop for Terminal {
    fn drop(&mut self) {
        restore();
    }
}
