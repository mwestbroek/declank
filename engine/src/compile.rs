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