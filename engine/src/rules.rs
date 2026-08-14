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

pub fn compile_literal(id: &str, rule: &LiteralRule) -> Vec<CompiledLiteral> {
    
}

pub fn expand_lemma(rule: &LemmaRule) -> Vec<LiteralRule> {

}


pub fn parse_template(pattern: &str, replacement: &str) -> Result<TemplateRule, TemplateValidationError> {

}


pub fn compile_template(id: &str, rule: TemplateRule) -> CompiledTemplate {

}



