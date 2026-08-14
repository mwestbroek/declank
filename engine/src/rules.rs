use crate::inflect::{noun_forms, verb_forms};

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
pub struct CompiledLiteral {
    pub id: String,
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

pub struct Hole {
    name: String,
}
pub enum ReplacementPart {
    Text(String),
    Hole(Hole),
}

pub struct TemplateRule {
    lead: String,
    pairs: Vec<(Hole, String)>,
    tail: Option<Hole>,
    replacement: Vec<ReplacementPart>,
}
pub struct CompiledTemplate {
    pub id: String,
    pub rule: TemplateRule,
}

pub enum TemplateValidationError {
    EmptyLead,
    NoHoles,
    AdjacentHoles,
    TooManyHoles,
    UnclosedBrace,
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


pub fn parse_template(pattern: &str, replacement: &str) -> Result<TemplateRule, TemplateValidationError> {

}


pub fn compile_template(id: &str, rule: TemplateRule) -> CompiledTemplate {

}



