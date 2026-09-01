use crate::{book::Book, rules::{LiteralRule, Rule, RuleKind, TemplateRule, expand_lemma}};

pub struct CompiledLiteral {
    pub id: String,
    pub pattern: String,
    pub replacement: String,
}

pub struct CompiledTemplate {
    pub id: String,
    pub rule: TemplateRule,
}

pub struct CompiledRules {
    pub literals: Vec<CompiledLiteral>,
    pub templates: Vec<CompiledTemplate>,
    pub enabled: bool,
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}
pub fn compile_literal(id: &str, rule: &LiteralRule) -> Vec<CompiledLiteral> {
    let mut rules = vec![CompiledLiteral {
        id: id.to_string(),
        pattern: rule.pattern.clone(),
        replacement: rule.replacement.clone(),
    }];

    let capitalised = capitalise(&rule.pattern);
    if capitalised != rule.pattern {
        rules.push(CompiledLiteral {
            id: id.to_string(),
            pattern: capitalised,
            replacement: capitalise(&rule.replacement),
        });
    }

    rules
}

pub fn compile_template(id: &str, rule: TemplateRule) -> CompiledTemplate {
    CompiledTemplate { id: id.to_string(), rule }
}



pub fn compile(rules: Vec<Rule>, book_enabled: bool) -> CompiledRules {
    let mut literals = Vec::new();
    let mut templates = Vec::new();

    for rule in rules {
        let Rule { id, enabled, kind } = rule;
        if !enabled {
            continue;
        }
        match kind {
            RuleKind::Lemma(rule) => {
                // parse into literal rules
                let expanded = expand_lemma(&rule);
                for rule in expanded {
                    literals.extend(compile_literal(&id, &rule))
                };
            },
            RuleKind::Literal(rule) => {
                literals.extend(compile_literal(&id, &rule));
            },
            RuleKind::Template(template) => {
                templates.push(compile_template(&id, template))
            }
        }
    }

    CompiledRules { literals, templates, enabled: book_enabled }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::book::deserialise_book;
    use crate::rewrite::rewrite;

    /// JSON in, compiled rules out. The whole pipeline bar the WASM boundary.
    fn build(json: &str) -> CompiledRules {
        let book = serde_json::from_str(json).expect("test book should parse");
        let crate::book::Book { rules, enabled, .. } = deserialise_book(book);
        compile(rules, enabled)
    }

    const LEMMAS: &str = r#"{
        "version": 1,
        "rules": [
            {"id":"utilise","kind":"lemma-verb","pattern":"utilise","replacement":"use"},
            {"id":"delve","kind":"lemma-verb","pattern":"delve","replacement":"look"},
            {"id":"intricacy","kind":"lemma-noun","pattern":"intricacy","replacement":"detail"}
        ]
    }"#;

    // ---------- lemma expansion ----------

    #[test]
    fn verb_lemma_expands_to_all_forms() {
        let r = build(LEMMAS);
        assert_eq!(rewrite("We utilise this.", &r), "We use this.");
        assert_eq!(rewrite("She utilises this.", &r), "She uses this.");
        assert_eq!(rewrite("They utilised this.", &r), "They used this.");
        assert_eq!(rewrite("We are utilising this.", &r), "We are using this.");
    }

    #[test]
    fn e_drop_in_participle() {
        // delve -> delving, not delveing
        let r = build(LEMMAS);
        assert_eq!(rewrite("We are delving here.", &r), "We are looking here.");
    }

    #[test]
    fn noun_lemma_gives_singular_and_plural() {
        let r = build(LEMMAS);
        assert_eq!(rewrite("The intricacy is clear.", &r), "The detail is clear.");
        assert_eq!(rewrite("The intricacies are clear.", &r), "The details are clear.");
    }

    #[test]
    fn every_compiled_form_carries_the_authoring_rule_id() {
        let r = build(LEMMAS);
        let utilise_forms: Vec<&str> = r
            .literals
            .iter()
            .filter(|l| l.id == "utilise")
            .map(|l| l.pattern.as_str())
            .collect();
        // Four forms, each in two cases.
        assert_eq!(utilise_forms.len(), 8, "got {utilise_forms:?}");
        assert!(utilise_forms.contains(&"utilise"));
        assert!(utilise_forms.contains(&"Utilise"));
        assert!(utilise_forms.contains(&"utilising"));
    }

    // ---------- case expansion ----------

    #[test]
    fn sentence_initial_capital_matches() {
        let r = build(LEMMAS);
        assert_eq!(rewrite("Utilise this.", &r), "Use this.");
        assert_eq!(rewrite("Utilising this works.", &r), "Using this works.");
    }

    #[test]
    fn all_caps_deliberately_does_not_match() {
        // Only two variants compile: as authored, and sentence case.
        let r = build(LEMMAS);
        assert_eq!(rewrite("UTILISE this.", &r), "UTILISE this.");
    }

    // ---------- multi-word literals ----------

    #[test]
    fn multi_word_literal_in_both_cases() {
        let r = build(r#"{
            "version": 1,
            "rules": [
                {"id":"fast-paced","kind":"literal","pattern":"in today's fast-paced world","replacement":"currently"}
            ]
        }"#);
        assert_eq!(
            rewrite("We know in today's fast-paced world things move.", &r),
            "We know currently things move."
        );
        assert_eq!(
            rewrite("In today's fast-paced world things move.", &r),
            "Currently things move."
        );
    }

    // ---------- known gaps ----------

    #[test]
    fn affixed_forms_do_not_match() {
        // Word boundaries are always on, so "underutilise" is untouched.
        let r = build(LEMMAS);
        assert_eq!(rewrite("Underutilise this.", &r), "Underutilise this.");
    }

    #[test]
    fn deletion_leaves_whitespace_for_the_tidy_step() {
        // No tidy step yet. Update this when whitespace normalisation lands.
        let r = build(r#"{
            "version": 1,
            "rules": [
                {"id":"crucial","kind":"literal","pattern":"it is crucial to note that","replacement":""}
            ]
        }"#);
        assert_eq!(
            rewrite("We know it is crucial to note that the system fails.", &r),
            "We know  the system fails."
        );
    }

    #[test]
    fn non_ascii_input_does_not_panic() {
        let r = build(LEMMAS);
        let s = "Wir müssen das naïve café 日本語 prüfen.";
        assert_eq!(rewrite(s, &r), s);
        assert_eq!(rewrite("Wir utilise café.", &r), "Wir use café.");
    }
}