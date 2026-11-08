//! The Key at the back of the book (docs/194): every exercise's answers,
//! parsed from the raw text — `UNIT n`, `e.m`, then `k answer / alternative`
//! with wrapped lines joined. The Key is never told to the student; it is
//! what the student is graded against.

use phient::prose::tokens;

/// One answer: its unit and its alternatives as tokens.
pub struct Answer {
    pub unit: usize,
    pub alternative: Vec<Vec<String>>,
}

/// Parses the Key from the raw book text (the LAST `Key to Exercises`
/// heading, so the contents line is skipped).
pub fn parse(raw: &str) -> Vec<Answer> {
    let start = raw.match_indices("Key to Exercises").map(|(i, _)| i).nth(1).unwrap_or(0);
    let mut out: Vec<Answer> = Vec::new();
    let mut unit = 0usize;
    let mut open: Option<String> = None;
    let close = |open: &mut Option<String>, out: &mut Vec<Answer>, unit: usize| {
        if let Some(text) = open.take() {
            let alternative: Vec<Vec<String>> = text
                .split(" / ")
                .map(|a| tokens(a).into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
                .filter(|a: &Vec<String>| !a.is_empty())
                .collect();
            if !alternative.is_empty() && unit > 0 {
                out.push(Answer { unit, alternative });
            }
        }
    };
    for line in raw[start..].lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(n) = t.strip_prefix("UNIT ").and_then(|n| n.trim().parse::<usize>().ok()) {
            close(&mut open, &mut out, unit);
            unit = n;
            continue;
        }
        if t.starts_with("Key to Additional") {
            close(&mut open, &mut out, unit);
            break;
        }
        if t.split_once('.').is_some_and(|(a, b)| a.parse::<usize>().is_ok() && b.parse::<usize>().is_ok()) {
            close(&mut open, &mut out, unit);
            continue;
        }
        let mut it = t.splitn(2, ' ');
        match (it.next().and_then(|k| k.parse::<usize>().ok()), it.next()) {
            (Some(_), Some(rest)) => {
                close(&mut open, &mut out, unit);
                open = Some(rest.to_string());
            }
            (Some(_), None) => {} // a page number
            _ => {
                if let Some(o) = open.as_mut() {
                    o.push(' ');
                    o.push_str(t);
                }
            }
        }
    }
    close(&mut open, &mut out, unit);
    out
}
