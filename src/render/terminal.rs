use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, queue,
    style::{Print, ResetColor},
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    io::{self, Stdout, Write},
    sync::atomic::{AtomicBool, Ordering},
};
static ACTIVE: AtomicBool = AtomicBool::new(false);
static KEYS: AtomicBool = AtomicBool::new(false);
pub struct Terminal {
    pub key_releases: bool,
    out: Stdout,
    previous: Vec<char>,
    width: usize,
    output: Vec<u8>,
}
impl Terminal {
    pub fn enter() -> io::Result<Self> {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            previous_hook(info);
        }));
        terminal::enable_raw_mode()?;
        ACTIVE.store(true, Ordering::SeqCst);
        let mut terminal = Self {
            key_releases: false,
            out: io::stdout(),
            previous: vec![],
            width: 0,
            output: vec![],
        };
        execute!(
            terminal.out,
            EnterAlternateScreen,
            Hide,
            DisableLineWrap,
            EnableMouseCapture,
            EnableFocusChange,
            ResetColor
        )?;
        terminal.key_releases = terminal::supports_keyboard_enhancement().unwrap_or(false);
        if terminal.key_releases {
            KEYS.store(true, Ordering::SeqCst);
            execute!(
                terminal.out,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                )
            )?;
        }
        Ok(terminal)
    }
    /// Verify actual terminal cursor advance: Unicode width tables alone cannot
    /// determine an emulator's ambiguous-width policy or font configuration.
    pub fn verify_hybrid_widths(&mut self) -> io::Result<()> {
        if terminal::size()?.0 < 2 {
            return Err(io::Error::other(
                "hybrid width verification needs at least two columns; use --glyph-set ascii",
            ));
        }
        for glyph in super::glyphs::HYBRID_GLYPHS {
            execute!(self.out, MoveTo(0, 0), Print(glyph))?;
            let position = crossterm::cursor::position().map_err(|e| {
                io::Error::other(format!(
                    "cannot verify hybrid glyph widths ({e}); use --glyph-set ascii"
                ))
            })?;
            if position != (1, 0) {
                return Err(io::Error::other(format!(
                    "glyph {glyph:?} did not advance exactly one column ({position:?}); use --glyph-set ascii"
                )));
            }
        }
        execute!(self.out, MoveTo(0, 0), Clear(ClearType::All))?;
        Ok(())
    }
    pub fn present(&mut self, cells: &[char], width: usize) -> io::Result<()> {
        if width == 0 {
            return Ok(());
        }
        self.output.clear();
        if self.previous.len() != cells.len() || self.width != width {
            self.previous = vec!['\0'; cells.len()];
            self.width = width;
            queue!(self.output, Clear(ClearType::All))?;
        }
        for (y, row) in cells.chunks(width).enumerate() {
            let old = &self.previous[y * width..y * width + row.len()];
            let Some(first) = row.iter().zip(old).position(|(a, b)| a != b) else {
                continue;
            };
            let last = row.iter().zip(old).rposition(|(a, b)| a != b).unwrap();
            queue!(self.output, MoveTo(first as u16, y as u16))?;
            encode_span(&mut self.output, &row[first..=last]);
        }
        self.out.write_all(&self.output)?;
        self.out.flush()?;
        self.previous.copy_from_slice(cells);
        Ok(())
    }
}
fn encode_span(output: &mut Vec<u8>, cells: &[char]) {
    let mut encoded = [0u8; 4];
    for c in cells {
        output.extend_from_slice(c.encode_utf8(&mut encoded).as_bytes());
    }
}
fn restore() {
    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let mut out = io::stdout();
    if KEYS.swap(false, Ordering::SeqCst) {
        let _ = execute!(out, PopKeyboardEnhancementFlags);
    }
    let _ = execute!(
        out,
        ResetColor,
        DisableFocusChange,
        DisableMouseCapture,
        EnableLineWrap,
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

#[cfg(test)]
mod tests {
    #[test]
    fn changed_span_is_valid_utf8() {
        let mut bytes = Vec::new();
        super::encode_span(&mut bytes, &['a', '│', '▒', '╲']);
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "a│▒╲");
    }
}
