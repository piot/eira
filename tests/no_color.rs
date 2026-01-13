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
fn test_without_color() {
    let source = source_from_raw(
        r#"1
fn main() {
    let x = undefined_value;
    println!("{}", x);
}
"#,
    );

    let mut section = SourceFileSection::new();
    section.labels.push(Label {
        start: Pos { x: 13, y: 3 },
        character_count: 15,
        color: Color::BrightGreen,
        text: "this variable is not defined".to_string(),
    });

    eprintln!("--- Testing WITHOUT color ---");
    let printer = Printer::without_color();
    section.draw(&source, &printer, stderr()).unwrap();
}

#[test]
fn test_with_color() {
    let source = source_from_raw(
        r#"1
fn main() {
    let x = undefined_value;
    println!("{}", x);
}
"#,
    );

    let mut section = SourceFileSection::new();
    section.labels.push(Label {
        start: Pos { x: 13, y: 3 },
        character_count: 15,
        color: Color::BrightRed,
        text: "this variable is not defined".to_string(),
    });

    eprintln!("--- Testing WITH color ---");
    let printer = Printer::new();
    section.draw(&source, &printer, stderr()).unwrap();
}
