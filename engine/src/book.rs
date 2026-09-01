use crate::rules::{
    LemmaKind, LemmaRule, LiteralRule, Rule, RuleKind, TemplateValidationError,
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
pub struct SerialisableBook {
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

#[derive(Serialize)]
pub struct RuleError {
    id: String,
    error: String,
}

pub struct Book {
    pub enabled: bool,
    pub rules: Vec<Rule>,
    pub errors: Vec<RuleError>,
}

pub fn deserialise_book(book: SerialisableBook) -> Book {
    let mut rules = Vec::new();
    let mut errors = Vec::new();

    for serialised_rule in book.rules {
        let rule_id = serialised_rule.id.clone();
        match deserialise_rule(serialised_rule) {
            Ok(rule) => rules.push(rule),
            Err(e) => errors.push(RuleError {
                id: rule_id,
                error: e.to_string(),
            }),
        };
    }

    Book {
        enabled: book.enabled,
        rules,
        errors,
    }
}