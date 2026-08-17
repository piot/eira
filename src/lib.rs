/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/eira
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use std::fmt::{self, Display, Formatter, Write};

pub use tinter::Color;
pub use tinter::Printer;

// Visual constants
const MIN_CONNECTOR_DASHES: usize = 2;
const MAX_CONNECTOR_DASHES: usize = 8;
const CONNECTOR_DASH_DIVISOR: usize = 4;

#[derive(Clone, PartialEq, Eq)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, PartialEq, Eq)]
pub struct PosSpan {
    pub pos: Pos,
    pub length: usize,
}

#[derive(PartialEq, Eq)]
pub struct ColoredSpan {
    pub pos: PosSpan,
    pub color: Color,
}

#[derive(PartialEq, Eq)]
pub struct Scope {
    pub start: PosSpan,
    pub end: PosSpan,
    pub text: String,
    pub color: Color,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Label {
    pub start: Pos,
    pub character_count: usize,
    pub text: String,
    pub color: Color,
}

pub trait SourceLines {
    fn get_line(&self, line_number: usize) -> Option<&str>;
}

// Scopes and Labels for a section of a source code file
pub struct SourceFileSection {
    pub scopes: Vec<Scope>,
    pub labels: Vec<Label>,
    pub tab_expansion_width: usize,
}

impl Default for SourceFileSection {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PrefixInfo<'a> {
    pub maximum_overlapping_scope_count: usize,
    pub active_scopes: &'a [&'a Scope],
    pub max_number_string_size: usize,
    line_number: Option<usize>,
}

impl SourceFileSection {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scopes: vec![],
            labels: vec![],
            tab_expansion_width: 4,
        }
    }

    /// Writes the number of specified spaces
    fn source_code_pad<W: Write>(count: usize, mut writer: W) -> fmt::Result {
        write!(writer, "{}", " ".repeat(count))
    }

    /// Writes the number of specified spaces
    fn scope_margin_pad<W: Write>(count: usize, mut writer: W) -> fmt::Result {
        write!(writer, "{}", " ".repeat(count))
    }

    /// Writes the number of specified spaces
    fn line_number_margin_pad<W: Write>(count: usize, mut writer: W) -> fmt::Result {
        write!(writer, "{}", " ".repeat(count))
    }

    /// Calculates which spans that are active for the specified source line.
    fn get_colored_spans_for_line(
        line_labels: &[&Label],
        scopes: &[&Scope],
        line_number: usize,
        source_line: &str,
        tab_width: usize,
    ) -> Vec<ColoredSpan> {
        let mut spans = Vec::new();

        // Add label spans with adjusted positions
        spans.extend(line_labels.iter().map(|label| {
            let visual_x =
                Self::convert_char_pos_to_visual_pos(source_line, label.start.x, tab_width);
            ColoredSpan {
                pos: PosSpan {
                    pos: Pos {
                        x: visual_x,
                        y: label.start.y,
                    },
                    length: label.character_count,
                },
                color: label.color,
            }
        }));

        // Add scope start spans with adjusted positions
        spans.extend(scopes.iter().filter_map(|scope| {
            if scope.start.pos.y == line_number {
                let visual_x =
                    Self::convert_char_pos_to_visual_pos(source_line, scope.start.pos.x, tab_width);
                Some(ColoredSpan {
                    pos: PosSpan {
                        pos: Pos {
                            x: visual_x,
                            y: scope.start.pos.y,
                        },
                        length: scope.start.length,
                    },
                    color: scope.color,
                })
            } else {
                None
            }
        }));

        // Add scope end spans with adjusted positions
        spans.extend(scopes.iter().filter_map(|scope| {
            if scope.end.pos.y == line_number {
                let visual_x =
                    Self::convert_char_pos_to_visual_pos(source_line, scope.end.pos.x, tab_width);
                Some(ColoredSpan {
                    pos: PosSpan {
                        pos: Pos {
                            x: visual_x,
                            y: scope.end.pos.y,
                        },
                        length: scope.end.length,
                    },
                    color: scope.color,
                })
            } else {
                None
            }
        }));

        spans
    }

    /// Converts tabs to spaces - each tab becomes exactly `tab_width` spaces.
    fn expand_tabs(source_line: &str, tab_width: usize) -> String {
        source_line.replace('\t', &" ".repeat(tab_width))
    }

    /// Converts a character position (where tab=1 char) to a visual position (where `tab=tab_width` chars).
    fn convert_char_pos_to_visual_pos(
        source_line: &str,
        char_pos: usize,
        tab_width: usize,
    ) -> usize {
        let chars: Vec<char> = source_line.chars().collect();
        let mut visual_pos = 0;

        for ch in chars
            .iter()
            .take(char_pos.saturating_sub(1).min(chars.len()))
        {
            if *ch == '\t' {
                visual_pos += tab_width;
            } else {
                visual_pos += 1;
            }
        }

        visual_pos + 1 // Convert to 1-based position
    }

    /// Writes a source line with syntax coloring based on colored spans.
    fn write_source_line<W: Write>(
        source_line: &str,
        colored_spans: &[ColoredSpan],
        tab_width: usize,
        printer: &Printer,
        mut writer: W,
    ) -> fmt::Result {
        let expanded_line = Self::expand_tabs(source_line, tab_width);
        let chars: Vec<char> = expanded_line.chars().collect();
        let mut current_pos = 0;

        while current_pos < chars.len() {
            let matching_span = colored_spans.iter().find(|span| {
                (current_pos + 1) >= span.pos.pos.x
                    && (current_pos + 1) < span.pos.pos.x + span.pos.length
            });

            let mut region_end = current_pos;
            while region_end < chars.len() {
                let next_span = colored_spans.iter().find(|span| {
                    (region_end + 1) >= span.pos.pos.x
                        && (region_end + 1) < span.pos.pos.x + span.pos.length
                });
                if next_span != matching_span {
                    break;
                }
                region_end += 1;
            }

            let text: String = chars[current_pos..region_end].iter().collect();
            if let Some(span) = matching_span {
                write!(writer, "{}", printer.color(span.color, &text))?;
            } else {
                write!(writer, "{text}")?;
            }

            current_pos = region_end;
        }

        writeln!(writer)?;
        Ok(())
    }

    /// Calculates which line numbers need to be displayed based on labels and scopes.
    #[must_use]
    pub fn calculate_source_lines_that_must_be_shown(&self) -> Vec<usize> {
        // Only filter out source code lines that will be referenced by scope or labels
        let mut lines_to_show: Vec<usize> = self
            .labels
            .iter()
            .map(|label| label.start.y)
            .chain(
                self.scopes
                    .iter()
                    .flat_map(|scope| vec![scope.start.pos.y, scope.end.pos.y]),
            )
            .collect();
        lines_to_show.sort_unstable();
        lines_to_show.dedup();
        lines_to_show
    }

    /// Sorts the labels and scopes by position for consistent rendering.
    pub fn layout(&mut self) {
        // Sort scopes by x position first, then by y position
        self.scopes
            .sort_by(|a, b| match a.start.pos.x.cmp(&b.start.pos.x) {
                std::cmp::Ordering::Equal => a.start.pos.y.cmp(&b.start.pos.y),
                other => other,
            });

        // Sort label positions by y first and then by x.
        self.labels.sort_by(|a, b| match a.start.y.cmp(&b.start.y) {
            std::cmp::Ordering::Equal => a.start.x.cmp(&b.start.x),
            other => other,
        });
    }

    /// Writes vertical bars for active scopes in the prefix area.
    fn write_scope_continuation<W: Write>(
        active_scopes: &[&Scope],
        max_scopes: usize,
        printer: &Printer,
        mut writer: W,
    ) -> fmt::Result {
        let mut sorted_scopes = active_scopes.to_vec();
        sorted_scopes.sort_by_key(|scope| scope.start.pos.x);

        for i in 0..max_scopes {
            if let Some(scope) = sorted_scopes.get(i) {
                write!(writer, "{}", printer.color(scope.color, "│"))?;
                Self::scope_margin_pad(3, &mut writer)?;
            } else {
                Self::scope_margin_pad(4, &mut writer)?;
            }
        }
        Ok(())
    }

    /// Writes the line number and separator prefix for each line.
    fn write_line_prefix<W: Write>(
        max_line_num_width: usize,
        line_number: Option<usize>,
        printer: &Printer,
        mut writer: W,
    ) -> fmt::Result {
        let number_string =
            line_number.map_or_else(String::new, |found_number| found_number.to_string());

        let padding = max_line_num_width - number_string.len();

        Self::line_number_margin_pad(padding, &mut writer)?;
        write!(
            writer,
            "{}",
            printer.color(Color::BrightBlack, &number_string)
        )?;
        Self::line_number_margin_pad(1, &mut writer)?;
        let separator = if line_number.is_some() { "|" } else { "·" };

        write!(writer, "{}", printer.color(Color::BrightBlack, separator))?;
        Self::line_number_margin_pad(1, &mut writer)?;

        Ok(())
    }

    /// Calculates the maximum number of overlapping scope items for the source block.
    /// This is needed to determine the correct padding width.
    #[must_use]
    pub fn calculate_max_overlapping_scopes(scopes: &[Scope]) -> usize {
        scopes.iter().fold(0, |max_count, scope| {
            let overlapping = scopes
                .iter()
                .filter(|other| {
                    scope.start.pos.y <= other.end.pos.y && other.start.pos.y <= scope.end.pos.y
                })
                .count();
            max_count.max(overlapping)
        })
    }

    /// Writes a source code line with all its prefixes (line number, scope bars).
    ///
    /// # Errors
    /// Returns an error if the line number is not provided in `PrefixInfo`,
    /// or if there are formatting errors during writing.
    pub fn write_source_line_with_prefixes(
        prefix_info: &PrefixInfo,
        labels: &[&Label],
        source_line: &str,
        tab_width: usize,
        printer: &Printer,
        mut writer: impl Write,
    ) -> fmt::Result {
        let current_line_number = prefix_info.line_number.ok_or(fmt::Error)?;
        Self::write_line_prefix(
            prefix_info.max_number_string_size,
            Some(current_line_number),
            printer,
            &mut writer,
        )?;
        for i in 0..prefix_info.maximum_overlapping_scope_count {
            if let Some(scope) = prefix_info.active_scopes.get(i) {
                let is_start = current_line_number == scope.start.pos.y;
                let is_end = current_line_number == scope.end.pos.y;
                let scope_line_prefix = if is_start {
                    "╭─▶"
                } else if is_end {
                    "├─▶"
                } else {
                    "│"
                };
                write!(writer, "{}", printer.color(scope.color, scope_line_prefix))?;
                let padding = if is_start || is_end { 1 } else { 3 };

                Self::scope_margin_pad(padding, &mut writer)?;
            } else {
                Self::scope_margin_pad(4, &mut writer)?;
            }
        }
        let colored_spans = Self::get_colored_spans_for_line(
            labels,
            prefix_info.active_scopes,
            current_line_number,
            source_line,
            tab_width,
        );
        Self::write_source_line(source_line, &colored_spans, tab_width, printer, &mut writer)
    }

    /// Writes the prefix for a line (line number and scope bars).
    ///
    /// # Errors
    /// Returns an error if there are formatting errors during writing.
    pub fn write_start_of_line_prefix(
        prefix: &PrefixInfo,
        printer: &Printer,
        mut writer: impl Write,
    ) -> fmt::Result {
        Self::write_line_prefix(
            prefix.max_number_string_size,
            prefix.line_number,
            printer,
            &mut writer,
        )?;

        Self::write_scope_continuation(
            prefix.active_scopes,
            prefix.maximum_overlapping_scope_count,
            printer,
            &mut writer,
        )
    }

    /// Writes underline markers (─┬─) beneath source code to indicate label positions.
    ///
    /// # Errors
    /// Returns an error if character_count is zero or if there are formatting errors during writing.
    pub fn write_underlines_for_upcoming_labels(
        prefix_info: &PrefixInfo,
        line_labels: &[&Label],
        source_line: &str,
        tab_width: usize,
        printer: &Printer,
        mut writer: impl Write,
    ) -> fmt::Result {
        Self::write_start_of_line_prefix(prefix_info, printer, &mut writer)?;

        let mut current_pos = 0;

        for label in line_labels.iter().rev() {
            let visual_x =
                Self::convert_char_pos_to_visual_pos(source_line, label.start.x, tab_width);

            if visual_x > current_pos {
                Self::source_code_pad(visual_x - 1 - current_pos, &mut writer)?;
            }

            if label.character_count == 0 {
                return Err(fmt::Error);
            }
            let middle = (label.character_count - 1) / 2;
            let underline: String = (0..label.character_count)
                .map(|i| if i == middle { '┬' } else { '─' })
                .collect();

            write!(writer, "{}", printer.color(label.color, &underline))?;

            current_pos = visual_x - 1 + label.character_count;
        }

        writeln!(writer)
    }

    /// Writes label text with connectors (╰──) pointing to their positions in the source.
    ///
    /// # Errors
    /// Returns an error if there are formatting errors during writing.
    pub fn write_labels(
        prefix_info: &PrefixInfo,
        line_labels: &[&Label],
        source_line: &str,
        tab_width: usize,
        printer: &Printer,
        mut writer: impl Write,
    ) -> fmt::Result {
        for (idx, label) in line_labels.iter().enumerate() {
            Self::write_start_of_line_prefix(prefix_info, printer, &mut writer)?;

            let mut current_pos = 0;

            // Draw vertical bars for all labels that will come after this one
            for future_label in line_labels.iter().skip(idx + 1) {
                let visual_x = Self::convert_char_pos_to_visual_pos(
                    source_line,
                    future_label.start.x,
                    tab_width,
                );
                let middle = (visual_x - 1) + (future_label.character_count - 1) / 2;
                Self::source_code_pad(middle - current_pos, &mut writer)?;
                write!(writer, "{}", printer.color(future_label.color, "│"))?;
                current_pos = middle + 1;
            }

            // TODO: Store the aligned position so it doesn't have to be calculated again.
            let visual_x =
                Self::convert_char_pos_to_visual_pos(source_line, label.start.x, tab_width);
            let middle = (visual_x - 1) + (label.character_count - 1) / 2;
            if middle > current_pos {
                Self::source_code_pad(middle - current_pos, &mut writer)?;
            }

            // Line length somewhat proportional to the span so it looks nicer
            let dash_count = (label.character_count / CONNECTOR_DASH_DIVISOR)
                .clamp(MIN_CONNECTOR_DASHES, MAX_CONNECTOR_DASHES);
            let connector = format!("╰{}", "─".repeat(dash_count));
            let label_line = format!("{} {}", printer.color(label.color, &connector), label.text);
            write!(writer, "{label_line}")?;

            writeln!(writer)?;
        }
        Ok(())
    }

    /// Writes text descriptions for scopes that end on the current line.
    ///
    /// # Errors
    /// Returns an error if there are formatting errors during writing.
    pub fn write_text_for_ending_scopes(
        prefix_info: &PrefixInfo,
        active_scopes: &[&Scope],
        line_number: usize, // Line number is provided, since the prefix_info line_number is `None`.
        printer: &Printer,
        mut writer: impl Write,
    ) -> fmt::Result {
        for scope in active_scopes {
            if scope.end.pos.y == line_number {
                Self::write_start_of_line_prefix(prefix_info, printer, &mut writer)?;
                writeln!(writer)?;

                Self::write_line_prefix(
                    prefix_info.max_number_string_size,
                    None,
                    printer,
                    &mut writer,
                )?;

                for i in 0..prefix_info.maximum_overlapping_scope_count {
                    if let Some(s) = active_scopes.get(i) {
                        if s == scope {
                            write!(writer, "{}", printer.color(s.color, "╰─── "))?;
                            break; // stop writing since we are on the scope text we should print
                        }
                        write!(writer, "{}", printer.color(s.color, "│"))?;
                        Self::scope_margin_pad(3, &mut writer)?;
                    } else {
                        write!(writer, "    ")?;
                    }
                }
                writeln!(writer, "{}", printer.color(scope.color, &scope.text))?;
            }
        }

        Ok(())
    }

    /// Renders the complete source file section with all labels and scopes.
    ///
    /// # Errors
    /// Returns an error if a source line cannot be found for a line number,
    /// or if there are formatting errors during writing.
    pub fn draw<W: Write, S: SourceLines>(
        &self,
        source: &S,
        printer: &Printer,
        mut writer: W,
    ) -> fmt::Result {
        let line_numbers_to_show = self.calculate_source_lines_that_must_be_shown();

        let max_overlapping_scopes_count = Self::calculate_max_overlapping_scopes(&self.scopes);

        let max_line_number_width = line_numbers_to_show
            .iter()
            .max()
            .map_or(0, |&max_line| max_line.to_string().len());

        for &line_number in &line_numbers_to_show {
            let source_line = source.get_line(line_number).ok_or(fmt::Error)?;

            // Get active scopes for source line (includes end line)
            let active_scopes: Vec<_> = self
                .scopes
                .iter()
                .filter(|scope| scope.start.pos.y <= line_number && line_number <= scope.end.pos.y)
                .collect();

            // Get labels for the current line and sort labels by x position in reverse order (right to left)
            let mut line_labels: Vec<_> = self
                .labels
                .iter()
                .filter(|label| label.start.y == line_number)
                .collect();
            line_labels.sort_by_key(|label| std::cmp::Reverse(label.start.x));

            let mut prefix_info = PrefixInfo {
                maximum_overlapping_scope_count: max_overlapping_scopes_count,
                active_scopes: &active_scopes,
                max_number_string_size: max_line_number_width,
                line_number: Some(line_number),
            };

            Self::write_source_line_with_prefixes(
                &prefix_info,
                &line_labels,
                source_line,
                self.tab_expansion_width,
                printer,
                &mut writer,
            )?;

            prefix_info.line_number = None; // only use line number when writing source lines

            Self::write_underlines_for_upcoming_labels(
                &prefix_info,
                &line_labels,
                source_line,
                self.tab_expansion_width,
                printer,
                &mut writer,
            )?;

            Self::write_labels(
                &prefix_info,
                &line_labels,
                source_line,
                self.tab_expansion_width,
                printer,
                &mut writer,
            )?;

            Self::write_text_for_ending_scopes(
                &prefix_info,
                &active_scopes,
                line_number,
                printer,
                &mut writer,
            )?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    Help, // Give extra information about an error. Maybe should be included in each warning and error?
    Idiom, // Suggests an idiomatic style or design choice.
    Lint, // Suggests removing redundant or unnecessary code.
    Advisory, // Warns about a concrete cost or risk without indicating invalid code.
    Note, // Extra context. Maybe should be included in each warning and error?
    Warning,
    Hint, // Styling, formatting, should maybe not be called hint? it is usually not reflected in AST changes, just whitespace and similar
    Error,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = match self {
            Self::Help => "help",
            Self::Idiom => "idiom",
            Self::Lint => "lint",
            Self::Advisory => "advisory",
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Hint => "hint",
            Self::Error => "error",
        };
        write!(f, "{prefix}")
    }
}

pub struct Header<C: Display> {
    pub header_kind: Kind,
    pub code: C,
    pub code_prefix: String,
    pub message: String,
}

impl<C: Display> Header<C> {
    const fn color_for_kind(kind: Kind) -> Color {
        match kind {
            Kind::Help => Color::BrightBlue,
            Kind::Idiom => Color::BrightCyan,
            Kind::Lint => Color::BrightBlack,
            Kind::Hint => Color::BrightBlack,
            Kind::Advisory => Color::BrightGreen,
            Kind::Note => Color::BrightBlack,
            Kind::Warning => Color::BrightYellow,
            Kind::Error => Color::BrightRed,
        }
    }

    /// Writes the error/warning header with code and message.
    ///
    /// # Errors
    /// Returns an error if there are formatting errors during writing.
    pub fn write<W: Write>(&self, printer: &Printer, mut writer: W) -> fmt::Result {
        write!(
            writer,
            "{}",
            printer.color(Self::color_for_kind(self.header_kind), &self.header_kind)
        )?;
        write!(
            writer,
            "[{}{}]",
            printer.color(Color::White, &self.code_prefix),
            printer.color(Color::Blue, &self.code)
        )?;
        write!(writer, ": ")?;
        write!(writer, "{}", &self.message)?;
        writeln!(writer)
    }
}

pub struct FileSpanMessage;

impl FileSpanMessage {
    /// Writes the file location pointer (e.g., " --> file.txt:3:10").
    ///
    /// # Errors
    /// Returns an error if there are formatting errors during writing.
    pub fn write<W: Write>(
        relative_file_name: &str,
        pos_span: &PosSpan,
        printer: &Printer,
        mut writer: W,
    ) -> fmt::Result {
        write!(writer, "  --> ")?;
        write!(
            writer,
            "{}",
            printer.color(Color::BrightCyan, relative_file_name)
        )?;
        write!(
            writer,
            ":{}:{}",
            printer.color(Color::BrightMagenta, &pos_span.pos.y),
            printer.color(Color::BrightMagenta, &pos_span.pos.x),
        )?;
        writeln!(writer)?;
        writeln!(writer)
    }
}
