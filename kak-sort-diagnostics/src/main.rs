use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, Read};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum DiagnosticType {
    Error = 0,
    Warning = 1,
    Hint = 2,
}

impl DiagnosticType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => DiagnosticType::Error,
            "warning" => DiagnosticType::Warning,
            "hint" => DiagnosticType::Hint,
            _ => DiagnosticType::Hint,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct DiagnosticGroup {
    file: String,
    line: usize,
    column: usize,
    diag_type: DiagnosticType,
    lines: Vec<String>,
    diags_in_file: usize,
}

impl Ord for DiagnosticGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.diag_type
            .cmp(&other.diag_type)
            .then(self.diags_in_file.cmp(&other.diags_in_file))
            .then(self.line.cmp(&other.line))
            .then(self.column.cmp(&other.column))
    }
}

impl PartialOrd for DiagnosticGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Regex to match "FILE:LINE:COLUMN: [TYPE]"
    // Example: "src/main.rs:10:5: [error]"
    let header_re = Regex::new(r"(?i)^([^:]+):(\d+):(\d+): \[(error|warning|hint)\]").unwrap();

    let mut groups: Vec<DiagnosticGroup> = Vec::new();
    let mut current_group: Option<DiagnosticGroup> = None;
    let mut groups_file_counter: HashMap<String, usize> = HashMap::new();

    for line in input.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let (prefix, processed) = process_line(trimmed);

        if let Some(caps) = header_re.captures(trimmed) {
            if let Some(group) = current_group.take() {
                groups.push(group);
            }

            let file = caps.get(1).unwrap().as_str().to_string();
            let line_num = caps.get(2).unwrap().as_str().parse::<usize>().unwrap_or(0);
            let col_num = caps.get(3).unwrap().as_str().parse::<usize>().unwrap_or(0);
            let diag_type = DiagnosticType::from_str(caps.get(4).unwrap().as_str());

            current_group = Some(DiagnosticGroup {
                file: file.clone(),
                line: line_num,
                column: col_num,
                diag_type,
                lines: vec![prefix, processed],
                diags_in_file: 0,
            });
            let cnt: &mut usize = groups_file_counter.entry(file.clone()).or_insert(0);
            *cnt += 1;
        } else if let Some(ref mut group) = current_group {
            group.lines.push(processed);
        }
    }

    if let Some(group) = current_group {
        groups.push(group);
    }
    for group in groups.iter_mut() {
        group.diags_in_file = groups_file_counter
            .get(&group.file)
            .copied()
            .unwrap_or(0);
    }

    groups.sort();

    for (i, group) in groups.iter().enumerate() {
        if i > 0 {
            println!();
        }
        for line in &group.lines {
            println!("{}", line);
        }
    }

    Ok(())
}

fn process_line(trimmed: &str) -> (String, String) {
    let process_re = Regex::new(r"(?i)^([^:]+):(\d+):(\d+): (.*?]) (.*)$").unwrap();
    if let Some(caps) = process_re.captures(trimmed) {
        let file: String = caps.get(1).unwrap().as_str().into();
        let line: String = caps.get(2).unwrap().as_str().into();
        let column: String = caps.get(3).unwrap().as_str().into();
        let prefix: String = caps.get(4).unwrap().as_str().into();
        let content: String = caps.get(5).unwrap().as_str().into();
        let line_prefix = format!("{file}:{line}:{column}");
        let content = content
            .split("␊")
            .map(|line| format!("{line_prefix}: {line}"))
            .collect::<Vec<_>>()
            .join("\n");

        (prefix, content)
    } else {
        (String::new(), String::from(trimmed))
    }
}
