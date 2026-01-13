/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/eira
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
use eira::{
    Color, FileSpanMessage, Header, Kind, Label, Pos, PosSpan, Printer, Scope, SourceFileSection,
    SourceLines,
};
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
fn one_label() {
    let source = source_from_raw(
        r#"1
fn main() {
    let x = undefined_value;
    println!("{}", x);
}
"#,
    );

    let mut l = SourceFileSection::new();
    l.labels.push(Label {
        start: Pos { x: 13, y: 3 },
        character_count: 15,
        color: Color::BrightGreen,
        text: "this variable is not defined".to_string(),
    });

    let printer = Printer::new();
    l.draw(&source, &printer, stderr()).unwrap();
}

#[test]
fn two_labels() {
    let source = source_from_raw(
        r#"1
fn main() {
    let x = undefined_value;
    println!("{}", x);
}
"#,
    );

    let mut l = SourceFileSection::new();
    l.labels.push(Label {
        start: Pos { x: 13, y: 3 },
        character_count: 15,
        color: Color::BrightYellow,

        text: "this variable is not defined".to_string(),
    });
    l.labels.push(Label {
        start: Pos { x: 5, y: 4 },
        character_count: 8,
        color: Color::BrightMagenta,
        text: "function println! is unknown".to_string(),
    });

    eprintln!("---------------");
    let printer = Printer::new();
    l.draw(&source, &printer, stderr()).unwrap();
}

#[test]
fn two_labels_on_same_line() {
    let source = source_from_raw(
        r#"1
fn main() {
    let x = undefined_value;
    println!("{}", x);
}
"#,
    );

    let mut l = SourceFileSection::new();
    l.labels.push(Label {
        start: Pos { x: 13, y: 3 },
        character_count: 15,
        color: Color::BrightMagenta,
        text: "this variable is not defined".to_string(),
    });
    l.labels.push(Label {
        start: Pos { x: 9, y: 3 },
        character_count: 1,
        color: Color::BrightYellow,
        text: "not sure what 'x' is".to_string(),
    });
    l.labels.push(Label {
        start: Pos { x: 5, y: 4 },
        character_count: 8,
        color: Color::BrightCyan,
        text: "function println! is unknown".to_string(),
    });

    eprintln!("---------------");
    let printer = Printer::new();
    l.draw(&source, &printer, stderr()).unwrap();
}

#[test]
fn two_labels_on_same_line_scope() {
    let source = source_from_raw(
        r#"1
fn main() {
    let x = undefined_value;
    if x {
       skipped_line()
       println!("{}", x);
    }
    another line
    and_another
}
"#,
    );

    let header = Header {
        header_kind: Kind::Error,
        code: 2044,
        code_prefix: "".to_string(),
        message: "Illegal symbol for the type".to_string(),
    };
    let printer = Printer::new();
    header
        .write(&printer, stderr())
        .expect("header should work");

    FileSpanMessage::write(
        "imaginary/path/render.swamp",
        &PosSpan {
            pos: Pos { x: 14, y: 1 },
            length: 13,
        },
        &printer,
        stderr(),
    )
    .expect("filespan message should work");

    let mut l = SourceFileSection::new();

    l.scopes.push(Scope {
        start: PosSpan {
            pos: Pos { x: 4, y: 4 },
            length: 8,
        },
        end: PosSpan {
            pos: Pos { x: 5, y: 7 },
            length: 1,
        },
        color: Color::Red,
        text: "If scope is here".to_string(),
    });

    l.scopes.push(Scope {
        start: PosSpan {
            pos: Pos { x: 4, y: 2 },
            length: 8,
        },
        end: PosSpan {
            pos: Pos { x: 1, y: 10 },
            length: 1,
        },
        color: Color::Green,
        text: "this is the scope".to_string(),
    });

    let label_color = Color::BrightMagenta;

    let unknown_function_message = format!(
        "{}{}{}",
        "function '",
        tinter::bright_blue("println!"),
        "' is unknown"
    );

    l.labels.push(Label {
        start: Pos { x: 8, y: 6 },
        character_count: 8,
        color: Color::BrightBlue,
        text: unknown_function_message,
    });

    let variable_message = format!(
        "{}{}{}",
        "Variable '",
        tinter::color(label_color, "undefined_value"),
        "' defined"
    );

    l.labels.push(Label {
        start: Pos { x: 13, y: 3 },
        character_count: 15,
        color: label_color,
        text: variable_message,
    });

    l.labels.push(Label {
        start: Pos { x: 9, y: 3 },
        character_count: 1,
        color: Color::BrightCyan,
        text: "not sure what 'x' is".to_string(),
    });

    l.layout();
    l.draw(&source, &printer, stderr()).unwrap();
}
