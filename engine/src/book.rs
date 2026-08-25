use crate::rules::{LemmaKind, LemmaRule, LiteralRule, Rule, RuleKind, parse_template};

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