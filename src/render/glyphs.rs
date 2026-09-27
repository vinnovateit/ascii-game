//! Output vocabulary only. The accepted ASCII mapper remains the canonical input.
use super::ascii::hash;
use unicode_width::UnicodeWidthChar;
pub const HYBRID_GLYPHS: [char; 8] = ['░', '▒', '▓', '█', '─', '│', '╱', '╲'];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlyphSet {
    #[default]
    Ascii,
    Hybrid,
}
impl GlyphSet {
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "ascii" => Ok(Self::Ascii),
            "hybrid" => Ok(Self::Hybrid),
            _ => Err("glyph-set must be ascii or hybrid".into()),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::Hybrid => "hybrid",
        }
    }
    pub fn validate(self) -> Result<(), String> {
        if self == Self::Hybrid {
            for glyph in HYBRID_GLYPHS {
                if glyph.width() != Some(1) {
                    return Err(format!("hybrid glyph {glyph:?} is not single-column"));
                }
            }
        }
        Ok(())
    }
    pub fn resolve(self, ascii: u8, x: usize, y: usize) -> char {
        if self == Self::Ascii {
            return ascii as char;
        }
        let h = hash(x.wrapping_add(1777), y.wrapping_add(911));
        match ascii {
            b'|' => '│',
            b'/' => '╱',
            b'\\' => '╲',
            b'=' => '─',
            b':' if h & 31 == 0 => '░',
            b'x' | b'*' if h & 31 == 0 => '▒',
            b'#' | b'%' if h & 31 == 0 => '▓',
            b'@' if h & 63 == 0 => '█',
            _ => ascii as char,
        }
    }
    pub fn resolve_frame(self, ascii: &[u8], width: usize, display: &mut [char]) {
        assert_eq!(ascii.len(), display.len());
        for (i, (&source, destination)) in ascii.iter().zip(display).enumerate() {
            *destination = self.resolve(source, i % width, i / width);
        }
    }
}
pub fn text(cells: &[char], width: usize) -> String {
    let mut result = String::with_capacity(cells.len() + cells.len() / width);
    for row in cells.chunks(width) {
        result.extend(row);
        result.push('\n');
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whitelist_is_single_column_and_all_glyphs_are_reachable() {
        GlyphSet::Hybrid.validate().unwrap();
        let mut found = std::collections::HashSet::new();
        for y in 0..8 {
            for x in 0..1024 {
                for source in 32..=126 {
                    let c = GlyphSet::Hybrid.resolve(source, x, y);
                    assert_eq!(c.width(), Some(1));
                    assert_eq!(source == b' ', c == ' ');
                    if !c.is_ascii() {
                        assert!(HYBRID_GLYPHS.contains(&c));
                        found.insert(c);
                    }
                }
            }
        }
        assert_eq!(found.len(), 8);
    }
    #[test]
    fn ascii_is_an_identity_resolver() {
        for c in 32..=126 {
            assert_eq!(GlyphSet::Ascii.resolve(c, 4, 7), c as char);
        }
    }
    #[test]
    fn unicode_rows_count_cells_not_utf8_bytes() {
        let cells = ['│', ' ', '█', '─', '╱', '╲'];
        let frame = text(&cells, 3);
        assert_eq!(frame.lines().count(), 2);
        assert!(frame.lines().all(|row| row.chars().count() == 3));
        assert!(frame.len() > cells.len() + 2);
    }
}
