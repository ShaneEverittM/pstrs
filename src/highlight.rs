//! Utilities for syntax highlighting.
//!
//! Encapsulates the [syntect] library, and its syntax and theme files.
//! So that users don't have to worry about storing global syntax and theme
//! sets.

use lazy_static::lazy_static;
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped as escape, LinesWithEndings},
};

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

/// Apply syntax highlighting to a string of content.
///
/// The content may be multi-line.
/// If an error occurs, then no highlighting is performed.
pub fn highlight(content: &str, syntax: &str, theme: &str) -> String {
    // Get requested syntax, or no syntax.
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(syntax)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    // Get requested theme, or default theme.
    let theme = THEME_SET
        .themes
        .get(theme)
        .cloned()
        .unwrap_or_else(Theme::default);

    // Make a highlighter for our syntax and theme.
    let mut highlighter = HighlightLines::new(syntax, &theme);

    // Make a closure to process each line.
    let process_line = |line| match highlighter.highlight_line(line, &SYNTAX_SET) {
        Ok(ranges) => escape(&ranges[..], false) + "\x1b[0m",
        Err(_) => line.to_string(),
    };

    // Map lines of the content to highlighted lines, then join to string.
    LinesWithEndings::from(content).map(process_line).collect()
}
