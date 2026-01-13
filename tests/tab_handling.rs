/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/eira
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use eira::{Color, Label, Pos, Printer, SourceFileSection, SourceLines};
use std::io::stderr;

struct TestSource {
    lines: Vec<String>,
}

impl SourceLines for TestSource {
    fn get_line(&self, line_number: usize) -> Option<&str> {
        self.lines.get(line_number - 1).map(String::as_str)
    }
}

fn source_from_raw(raw: &str) -> TestSource {
    TestSource {
        lines: raw.lines().map(String::from).collect(),
    }
}

#[test]
fn test_tab_expansion() {
    // Source with tabs - tabs are represented as actual tab characters here
    let source = source_from_raw("fn main() {\n\tlet x = 10;\n\tprintln!(\"{}\", x);\n}");

    let mut section = SourceFileSection::new();

    // Add a label pointing to the variable x using CHARACTER position (where tab=1 char)
    // Character positions: tab=1, 'l'=2, 'e'=3, 't'=4, ' '=5, 'x'=6
    section.labels.push(Label {
        start: Pos { x: 6, y: 2 },
        character_count: 1,
        color: Color::BrightYellow,
        text: "variable x (char pos 6)".to_string(),
    });

    eprintln!("--- Testing tab expansion at start of line ---");
    let printer = Printer::new();
    section.draw(&source, &printer, stderr()).unwrap();
}

#[test]
fn test_tab_in_middle_of_line() {
    // Test with tab in the middle of the line
    // "let x =\t10;" where tab is at character position 8
    let source = source_from_raw("fn main() {\n\tlet x =\t10;\n}");

    let mut section = SourceFileSection::new();

    // Character positions: tab=1, 'l'=2, 'e'=3, 't'=4, ' '=5, 'x'=6, ' '=7, '='=8, tab=9, '1'=10
    // Point to the '1' at character position 10
    section.labels.push(Label {
        start: Pos { x: 10, y: 2 },
        character_count: 2,
        color: Color::BrightRed,
        text: "number 10 (char pos 10)".to_string(),
    });

    eprintln!("--- Testing tab in middle of line ---");
    let printer = Printer::new();
    section.draw(&source, &printer, stderr()).unwrap();
}

#[test]
fn test_missing_source_line() {
    let source = source_from_raw("line 1\nline 2\nline 3");

    let mut section = SourceFileSection::new();

    // Add a label for line 10 which doesn't exist
    section.labels.push(Label {
        start: Pos { x: 1, y: 10 },
        character_count: 5,
        color: Color::BrightRed,
        text: "this line doesn't exist".to_string(),
    });

    eprintln!("--- Testing missing source line handling ---");
    let printer = Printer::new();
    let result = section.draw(&source, &printer, stderr());

    // This should return an error instead of panicking
    assert!(result.is_err());
    if let Err(e) = result {
        eprintln!("Expected error: {}", e);
        assert!(e.to_string().contains("Source line 10 not found"));
    }
}
