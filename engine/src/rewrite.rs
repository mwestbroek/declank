use crate::compile::CompiledLiteral;

pub struct Match {
    pub start: usize,
    pub end: usize,
    pub rule_id: String,
    pub replacement: String,
}

fn has_word_boundaries(sentence: &str, start: usize, end: usize) -> bool {
    let before_ok = sentence[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric());
    let after_ok = sentence[end..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_alphanumeric());
    before_ok && after_ok
}

fn find_literal_matches(sentence: &str, rules: &[CompiledLiteral]) -> Vec<Match> {
    let mut matches = Vec::new();
    for rule in rules {
        for (start_idx, pat) in sentence.match_indices(&rule.pattern) {
            let end_idx = start_idx + pat.len();
            if !has_word_boundaries(sentence, start_idx, end_idx) {
                continue;
            }
            matches.push(Match {
                start: start_idx,
                end: end_idx,
                rule_id: rule.id.clone(),
                replacement: rule.replacement.clone(),
            });
        }
    }

    matches
}

fn select(mut matches: Vec<Match>) -> Vec<Match> {
    // Sort matches by start index (asc) and length (desc)
    matches.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| (b.end - b.start).cmp(&(a.end - a.start)))
    });

    let mut kept: Vec<Match> = Vec::new();
    let mut cursor = 0;

    // Only keep a match if it starts at or after where the last one ended
    for m in matches {
        if m.start < cursor {
            continue;
        }
        cursor = m.end;
        kept.push(m);
    }

    kept
}

fn apply(sentence: &str, matches: &[Match]) -> String {
    let mut rewritten = String::new();
    let mut copied_up_to_idx = 0;
    for m in matches {
        rewritten.push_str(&sentence[copied_up_to_idx..m.start]);
        rewritten.push_str(&m.replacement);
        copied_up_to_idx = m.end;
    }
    rewritten.push_str(&sentence[copied_up_to_idx..]);

    rewritten
}

pub fn rewrite(sentence: &str, rules: &[CompiledLiteral]) -> String {
    let matches = find_literal_matches(sentence, rules);
    let sorted_matches = select(matches);
    apply(sentence, &sorted_matches)
}