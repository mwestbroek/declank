use crate::rules::{
    LemmaKind, LemmaRule, LiteralRule, Rule, RuleKind, TemplateRule, TemplateValidationError,
    parse_template,
};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum SerialisableRuleKind {
    Literal {
        pattern: String,
        replacement: String,
    },
    LemmaVerb {
        pattern: String,
        replacement: String,
    },
    LemmaNoun {
        pattern: String,
        replacement: String,
    },
    Template {
        pattern: String,
        replacement: String,
    },
}

#[derive(Serialize, Deserialize)]
struct SerialisableRule {
    id: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(flatten)]
    kind: SerialisableRuleKind,
}

#[derive(Serialize, Deserialize)]
struct SerialisableBook {
    // TODO do we actually need version
    version: u32,
    #[serde(default = "default_true")]
    enabled: bool,
    rules: Vec<SerialisableRule>,
}

fn deserialise_rule_kind(
    serialised: SerialisableRuleKind,
) -> Result<RuleKind, TemplateValidationError> {
    match serialised {
        SerialisableRuleKind::Literal {
            pattern,
            replacement,
        } => Ok(RuleKind::Literal(LiteralRule {
            pattern,
            replacement,
        })),
        SerialisableRuleKind::LemmaNoun {
            pattern,
            replacement,
        } => Ok(RuleKind::Lemma(LemmaRule {
            pattern,
            replacement,
            kind: LemmaKind::Noun,
        })),
        SerialisableRuleKind::LemmaVerb {
            pattern,
            replacement,
        } => Ok(RuleKind::Lemma(LemmaRule {
            pattern,
            replacement,
            kind: LemmaKind::Verb,
        })),
        SerialisableRuleKind::Template {
            pattern,
            replacement,
        } => parse_template(&pattern, &replacement).map(RuleKind::Template),
    }
}

fn deserialise_rule(serialised: SerialisableRule) -> Result<Rule, TemplateValidationError> {
    let kind = deserialise_rule_kind(serialised.kind)?;
    Ok(Rule {
        id: serialised.id,
        enabled: serialised.enabled,
        kind,
    })
}

struct RuleError {
    id: String,
    error: TemplateValidationError,
}

struct Book {
    enabled: bool,
    rules: Vec<Rule>,
    errors: Vec<RuleError>,
}

fn deserialise_book(book: SerialisableBook) -> Book {
    let mut rules = Vec::new();
    let mut errors = Vec::new();

    for serialised_rule in book.rules {
        let rule_id = serialised_rule.id.clone();
        match deserialise_rule(serialised_rule) {
            Ok(rule) => rules.push(rule),
            Err(e) => errors.push(RuleError {
                id: rule_id,
                error: e,
            }),
        };
    }

    Book {
        enabled: book.enabled,
        rules,
        errors,
    }
}

/// The rule book. Hard-coded for now
/// Panics if a template fails validation. That is a programming error while the
/// book is a constant, and should become a proper error path once rules can be
/// authored at runtime.
pub fn rule_book() -> Vec<Rule> {
    vec![
        // ---------- lemmas ----------
        lemma("utilise", "utilise", "use", LemmaKind::Verb),
        lemma("delve", "delve", "look", LemmaKind::Verb),
        lemma("leverage", "leverage", "use", LemmaKind::Verb),
        lemma("showcase", "showcase", "show", LemmaKind::Verb),
        lemma("intricacy", "intricacy", "detail", LemmaKind::Noun),
        // ---------- literals ----------
        literal("fast-paced", "in today's fast-paced world", "currently"),
        literal("landscape", "the ever-evolving landscape of", "the"),
        literal("crucial", "it is crucial to note that", ""),
        literal("tapestry", "rich tapestry of", "range of"),
        // ---------- templates ----------
        template("not-just", "it's not just {A}, it's {B}", "it's {B}"),
        template("in-question", "the {A} in question", "the {A}"),
    ]
}

fn literal(id: &str, pattern: &str, replacement: &str) -> Rule {
    Rule {
        id: id.to_string(),
        enabled: true,
        kind: RuleKind::Literal(LiteralRule {
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
        }),
    }
}

fn lemma(id: &str, pattern: &str, replacement: &str, kind: LemmaKind) -> Rule {
    Rule {
        id: id.to_string(),
        enabled: true,
        kind: RuleKind::Lemma(LemmaRule {
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            kind,
        }),
    }
}

fn template(id: &str, pattern: &str, replacement: &str) -> Rule {
    Rule {
        id: id.to_string(),
        enabled: true,
        kind: RuleKind::Template(
            parse_template(pattern, replacement)
                .unwrap_or_else(|e| panic!("bad template rule {id:?}: {e:?}")),
        ),
    }
}
