use anyhow::{Context, Result, anyhow};
use std::{fmt::Display, str::FromStr};


#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
struct LineSpec {
    line: usize,
    column: usize,
}

const REPLACEMENT: &str = "...";
#[derive(Debug, Clone)]
struct Fold {
    start: LineSpec,
    end: LineSpec,
    replacement: String,
}

#[derive(Debug)]
struct Range {
    start: LineSpec,
    end: LineSpec,
}
#[derive(Debug)]
struct FoldRanges {
    timestamp: usize,
    folds: Vec<Fold>,
}
struct SelectionRanges {
    ranges: Vec<Range>,
}
trait HasRange {
    fn start(&self) -> LineSpec;
    fn end(&self) -> LineSpec;
    fn overlap(&self, other: &impl HasRange) -> bool {
        if other.end() < self.start() {
            return false;
        };
        if other.start() > self.end() {
            return false;
        }
        true
    }
}
macro_rules! impl_has_range {
    ($forstruct:ident) => {
        impl HasRange for $forstruct {
            fn start(&self) -> LineSpec {
                self.start
            }
            fn end(&self) -> LineSpec {
                self.end
            }
        }
    };
}
impl_has_range!(Fold);
impl_has_range!(Range);

impl Display for FoldRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.folds.is_empty() {
            write!(f, "{}", self.timestamp)
        } else {
            write!(
                f,
                "{} {}",
                self.timestamp,
                self.folds
                    .iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
            )
        }
    }
}
impl Display for SelectionRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
        self.ranges
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(" "))
    }
}
impl Display for Fold {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}|{}", self.start, self.end, self.replacement)
    }
}
impl Display for Range {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.start, self.end)
    }
}
impl FromStr for LineSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (line_str, column_str) = s.split_once(".").context("Missing separator dot")?;
        let line: usize = usize::from_str(line_str)?;
        let column: usize = usize::from_str(column_str)?;
        Ok(LineSpec{line,column})
    }
}
impl FromStr for Range {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start_str, end_str) = s.split_once(",").context("Missing comma")?;
        let start: LineSpec = LineSpec::from_str(start_str)?;
        let end: LineSpec = LineSpec::from_str(end_str)?;
        Ok(Range { start, end })
    }
}
impl FromStr for Fold {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (range_str, fold_string) = s.split_once("|").context("Missing pipe char")?;
        let range: Range = Range::from_str(range_str)?;
        Ok(Fold {
            start: range.start(),
            end: range.end(),
            replacement: String::from(fold_string),
        })
    }
}
impl FromStr for FoldRanges {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((timestamp_str, folds_str)) = s.split_once(" ") {
            let timestamp = usize::from_str(timestamp_str)?;
            let folds: Vec<Fold> = folds_str
                .split_whitespace()
                .flat_map(Fold::from_str)
                .collect();
            Ok(Self { timestamp, folds })
        } else {
            let timestamp = usize::from_str(s)?;
            Ok(Self {
                timestamp,
                folds: vec![],
            })
        }
    }
}

impl FoldRanges {
    fn contains(&self, fold: &Fold) -> bool {
        for f in self.folds.iter() {
            if f.start == fold.start && f.end == fold.end {
                return true;
            }
        }
        false
    }
}

impl From<Range> for Fold {
    fn from(value: Range) -> Self {
        Self {
            start: value.start,
            end: value.end,
            replacement: String::from(REPLACEMENT),
        }
    }
}

impl FromStr for SelectionRanges {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ranges: Vec<Range> = s.split_whitespace().flat_map(Range::from_str).collect();
        Ok(Self { ranges })
    }
}

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let action = args.first().unwrap().to_lowercase();
    let result = match action.as_str() {
        "add" => add_fold(args_ref),
        "remove" => remove_fold(args_ref),
        _ => return Err(anyhow!("Unknown Action")),
    };
    result.context("Action Result")?;

    Ok(())
}
impl Display for LineSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.line, self.column)
    }
}

fn add_fold(args: Vec<&str>) -> Result<()> {
    if args.len() != 3 {
        return Err(anyhow!("Wrong arg number"));
    }
    let current_folds: FoldRanges = FoldRanges::from_str(args.get(1).unwrap())?;
    let selections: SelectionRanges = SelectionRanges::from_str(args.get(2).unwrap())?;

    let new_selections = selections
        .ranges
        .into_iter()
        .map(Fold::from)
        .filter(|f| !current_folds.contains(f))
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let out = if new_selections.len() > 1 {
        format!("{} {}", current_folds, new_selections)
    } else {
        format!("{}", current_folds)
    };
    println!("{}", out.trim());

    Ok(())
}

fn remove_fold(args: Vec<&str>) -> Result<()> {
    if args.len() != 3 {
        return Err(anyhow!("Wrong arg number"));
    }
    let current_folds: FoldRanges = FoldRanges::from_str(args.get(1).unwrap())?;
    let selections: SelectionRanges = SelectionRanges::from_str(args.get(2).unwrap())?;

    let mut remaining_folds: Vec<Fold> = Vec::new();
    for fold in current_folds.folds.iter() {
        let mut found = false;
        for selection in selections.ranges.iter() {
            if selection.overlap(fold) {
                found = true
            }
        }
        if !found {
            remaining_folds.push(fold.clone())
        }
    }
    let folds = FoldRanges {
        timestamp: current_folds.timestamp,
        folds: remaining_folds,
    };
    println!("{}", folds);
    Ok(())
}
