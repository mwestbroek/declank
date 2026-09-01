use serde::Serialize;

use crate::inflect::{noun_forms, verb_forms};
use std::fmt::Display;

const MAX_HOLES: usize = 3;

pub struct Rule {
    pub id: String, 
    pub enabled: bool,
    pub kind: RuleKind,
}

pub enum RuleKind {
    Literal(LiteralRule),
    Lemma(LemmaRule),
    Template(TemplateRule),
}

pub struct LiteralRule {
    pub pattern: String,
    pub replacement: String,
}

pub struct LemmaRule {
    pub pattern: String, 
    pub replacement: String,
    pub kind: LemmaKind,
}
pub enum LemmaKind {
    Noun,
    Verb,
}

enum Segment {
    Text(String),
    Hole(Hole),
}

pub struct Hole {
    pub name: String,
}
pub enum ReplacementPart {
    Text(String),
    Hole(Hole),
}

pub struct TemplateRule {
    pub lead: String,
    pub pairs: Vec<(Hole, String)>,
    pub tail: Option<Hole>,
    pub replacement: Vec<ReplacementPart>,
}

#[derive(Debug, Serialize)]
pub enum TemplateValidationError {
    EmptyTemplate,
    EmptyLead,
    NoHoles,
    AdjacentHoles,
    TooManyHoles,
    UnclosedBrace,
    UnknownHoleInReplacement(String),
}

impl Display for TemplateValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateValidationError::EmptyTemplate => write!(f, "template is empty"),
            TemplateValidationError::EmptyLead => write!(f, "lead is empty"),
            TemplateValidationError::NoHoles => write!(f, "no holes in template"),
            TemplateValidationError::AdjacentHoles => write!(f, "adjacent holes in template"),
            TemplateValidationError::TooManyHoles => write!(f, "too many holes in template"),
            TemplateValidationError::UnclosedBrace => write!(f, "unclosed brace in template"),
            TemplateValidationError::UnknownHoleInReplacement(name) => write!(f, "unknown hole in replacement: {}", name),
        }
    }
}

pub fn expand_lemma(rule: &LemmaRule) -> Vec<LiteralRule> {
    let mut rules = Vec::new();

    match &rule.kind {
        LemmaKind::Noun => {
            let pattern_forms = noun_forms(&rule.pattern); 
            let rep_forms = noun_forms(&rule.replacement);
            rules.push(LiteralRule { pattern: pattern_forms.singular, replacement: rep_forms.singular});
            rules.push(LiteralRule { pattern: pattern_forms.plural, replacement: rep_forms.plural});
            rules
        },
        LemmaKind::Verb => {
            let pattern_forms = verb_forms(&rule.pattern);
            let rep_forms = verb_forms(&rule.replacement);
            rules.push(LiteralRule { pattern: pattern_forms.base, replacement: rep_forms.base});
            rules.push(LiteralRule { pattern: pattern_forms.third_person, replacement: rep_forms.third_person});
            let collides = pattern_forms.past == pattern_forms.past_participle && rep_forms.past == rep_forms.past_participle;
            rules.push(LiteralRule { pattern: pattern_forms.past, replacement: rep_forms.past});
            if !collides {
                rules.push(LiteralRule { pattern: pattern_forms.past_participle, replacement: rep_forms.past_participle});
            }
            rules.push(LiteralRule { pattern: pattern_forms.present_participle, replacement: rep_forms.present_participle});
            rules
        },
    }

}

fn find_segments(raw: &str) -> Result<Vec<Segment>, TemplateValidationError> {
    let mut segments = Vec::new();
    let mut cursor = 0;

    loop {
        let rest = &raw[cursor..];

        let Some(open) = rest.find('{') else {
            // No more holes: whatever remains is text.
            if !rest.is_empty() {
                segments.push(Segment::Text(rest.to_string()));
            }
            break;
        };

        // Only look for the closer after the opener.
        let Some(close) = rest[open..].find('}') else {
            return Err(TemplateValidationError::UnclosedBrace);
        };
        let close = open + close;

        if open > 0 {
            segments.push(Segment::Text(rest[..open].to_string()));
        }

        // Braces are single-byte, so these offsets are safe.
        let name = rest[open + 1..close].to_string();
        segments.push(Segment::Hole(Hole { name }));

        cursor += close + 1;
    }

    Ok(segments)
}

fn check_all_replacement_parts_valid(
    pairs: &[(Hole, String)],
    tail: &Option<Hole>,
    replacements: &[ReplacementPart],
) -> Result<(), TemplateValidationError> {
    let pattern_names: Vec<&str> = pairs
        .iter()
        .map(|(hole, _)| hole.name.as_str())
        .chain(tail.iter().map(|hole| hole.name.as_str()))
        .collect();

    for part in replacements {
        if let ReplacementPart::Hole(hole) = part
            && !pattern_names.iter().any(|n| *n == hole.name) {
                return Err(TemplateValidationError::UnknownHoleInReplacement(hole.name.clone()));
            }
    }

    Ok(())
}

pub fn parse_template(pattern: &str, replacement: &str) -> Result<TemplateRule, TemplateValidationError> {
    let mut pattern_segments = find_segments(pattern)?.into_iter();
    let lead = match pattern_segments.next() {
        None => return Err(TemplateValidationError::EmptyTemplate),
        Some(Segment::Hole(_)) => return Err(TemplateValidationError::EmptyLead),
        Some(Segment::Text(text)) => text,
    };

    let mut num_holes = 0;
    let mut tail = None;
    let mut pairs = Vec::new();
    while let Some(segment) = pattern_segments.next() {
        let hole = match segment {
            Segment::Hole(h) => {
                num_holes += 1;
                h
            }
            Segment::Text(_) => unreachable!("segments alternate"),
        };

        match pattern_segments.next() {
            Some(Segment::Hole(_)) => return Err(TemplateValidationError::AdjacentHoles),
            Some(Segment::Text(text)) => pairs.push((hole, text)),
            None => {
                tail = Some(hole);
                break;
            }
        }
    } 

    if num_holes > MAX_HOLES {
        return Err(TemplateValidationError::TooManyHoles);
    } 
    if num_holes == 0 {
        return Err(TemplateValidationError::NoHoles);
    }


    let replacement_segments: Vec<ReplacementPart> = find_segments(replacement)?
        .into_iter()
        .map(|s| match s {
            Segment::Text(text) => ReplacementPart::Text(text),
            Segment::Hole(hole) => ReplacementPart::Hole(hole),
        })
        .collect();

    check_all_replacement_parts_valid(&pairs, &tail, &replacement_segments)?;
    
    Ok(TemplateRule{ lead, pairs, tail, replacement: replacement_segments })

}



