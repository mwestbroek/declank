use std::collections::HashMap;

use crate::{
    compile::{CompiledLiteral, CompiledTemplate},
    rules::ReplacementPart,
};

pub struct Match {
    pub start: usize,
    pub end: usize,
    pub rule_id: String,
    pub replacement: String,
}

fn has_leading_word_boundary(sentence: &str, start: usize) -> bool {
    sentence[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric())
}

fn has_word_boundaries(sentence: &str, start: usize, end: usize) -> bool {
    let after_ok = sentence[end..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_alphanumeric());
    has_leading_word_boundary(sentence, start) && after_ok
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

fn find_template_matches(sentence: &str, templates: &[CompiledTemplate]) -> Vec<Match> {
    let mut matches = Vec::new();
    for template in templates {
        // Find all the leads
        'outer: for (lead_idx, _) in sentence.match_indices(&template.rule.lead) {
            // Ensure the lead is not part of a word
            let lead = &template.rule.lead;
            if !has_leading_word_boundary(sentence, lead_idx) {
                continue;
            }
            let mut cursor = lead_idx + lead.len();
            // Ensure all text following holes is found
            let mut hole_contents = HashMap::new();
            for (hole, following_text) in &template.rule.pairs {
                match sentence[cursor..].find(following_text) {
                    None => {
                        continue 'outer;
                    }
                    Some(found_idx) => {
                        let found = cursor + found_idx; // found_idx is relative to the slice
                        hole_contents.insert(hole.name.as_str(), &sentence[cursor..found]);
                        cursor = found + following_text.len();
                    }
                };
            }
            if let Some(tail) = &template.rule.tail {
                hole_contents.insert(tail.name.as_str(), &sentence[cursor..]);
                cursor = sentence.len();
            };
            // The match spans from lead_idx to cursor
            let mut replacement = String::new();
            for replacement_part in &template.rule.replacement {
                match replacement_part {
                    ReplacementPart::Hole(hole) => replacement.push_str(
                        hole_contents
                            .get(hole.name.as_str())
                            .copied()
                            .unwrap_or_else(|| panic!("hole {:?} not captured", hole.name))
                    ),
                    ReplacementPart::Text(text) => replacement.push_str(text),
                };
            }
            matches.push(Match {
                start: lead_idx,
                end: cursor,
                rule_id: template.id.clone(),
                replacement,
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(id: &str, pattern: &str, replacement: &str) -> CompiledLiteral {
        CompiledLiteral {
            id: id.to_string(),
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
        }
    }

    // ---------- case A: literal replacement ----------

    #[test]
    fn case_a_two_matches_in_one_sentence() {
        let rules = [
            lit("utilise", "utilise", "use"),
            lit("delve", "delve", "look"),
        ];
        assert_eq!(
            rewrite("We utilise this to delve into the data.", &rules),
            "We use this to look into the data."
        );
    }

    #[test]
    fn same_rule_matches_repeatedly() {
        let rules = [lit("utilise", "utilise", "use")];
        assert_eq!(
            rewrite("We utilise it, they utilise it.", &rules),
            "We use it, they use it."
        );
    }

    // ---------- case C: overlap resolution ----------

    #[test]
    fn case_c_longest_match_wins_at_same_start() {
        let rules = [
            lit("delve", "delve", "look"),
            lit("delve-long", "delve into the intricacies of", "examine"),
        ];
        assert_eq!(
            rewrite("Let us delve into the intricacies of the problem.", &rules),
            "Let us examine the problem."
        );
    }

    #[test]
    fn rule_order_in_the_slice_does_not_matter() {
        // Same as above with the rules the other way round. Selection is
        // leftmost-longest, not first-declared-wins.
        let rules = [
            lit("delve-long", "delve into the intricacies of", "examine"),
            lit("delve", "delve", "look"),
        ];
        assert_eq!(
            rewrite("Let us delve into the intricacies of the problem.", &rules),
            "Let us examine the problem."
        );
    }

    #[test]
    fn leftmost_wins_when_matches_overlap_from_different_starts() {
        let rules = [lit("ab", "a b", "X"), lit("bc", "b c", "Y")];
        // "a b c": the a-b match starts first and consumes the b, so b-c is dropped.
        assert_eq!(rewrite("a b c", &rules), "X c");
    }

    // ---------- word boundaries ----------

    #[test]
    fn does_not_match_inside_words() {
        let rules = [lit("and", "and", "&")];
        assert_eq!(
            rewrite("It handles the standard and the rest.", &rules),
            "It handles the standard & the rest."
        );
    }

    #[test]
    fn matches_at_start_and_end_of_sentence() {
        let rules = [lit("and", "and", "&")];
        assert_eq!(rewrite("and", &rules), "&");
        assert_eq!(rewrite("and then", &rules), "& then");
        assert_eq!(rewrite("this and", &rules), "this &");
    }

    #[test]
    fn punctuation_counts_as_a_boundary() {
        let rules = [lit("and", "and", "&")];
        assert_eq!(rewrite("(and)", &rules), "(&)");
        assert_eq!(rewrite("and, then", &rules), "&, then");
    }

    #[test]
    fn multi_word_patterns_match() {
        let rules = [lit(
            "fast-paced",
            "in today's fast-paced world",
            "currently",
        )];
        assert_eq!(
            rewrite("In today's fast-paced world things move.", &rules),
            "In today's fast-paced world things move."
        );
        // Lowercase form matches; the capitalised variant is a separate
        // compiled rule that this test does not include.
        assert_eq!(
            rewrite("We know in today's fast-paced world things move.", &rules),
            "We know currently things move."
        );
    }

    // ---------- deletions ----------

    #[test]
    fn empty_replacement_deletes_and_leaves_a_double_space() {
        // The double space is deliberate. Whitespace normalisation is the tidy
        // step's job, not apply's.
        let rules = [lit("crucial", "it is crucial to note that", "")];
        assert_eq!(
            rewrite(
                "We know it is crucial to note that the system fails.",
                &rules
            ),
            "We know  the system fails."
        );
    }

    // ---------- degenerate inputs ----------

    #[test]
    fn no_matches_returns_the_sentence_unchanged() {
        let rules = [lit("utilise", "utilise", "use")];
        assert_eq!(rewrite("A normal sentence.", &rules), "A normal sentence.");
    }

    #[test]
    fn empty_sentence() {
        let rules = [lit("utilise", "utilise", "use")];
        assert_eq!(rewrite("", &rules), "");
    }

    #[test]
    fn empty_rule_set() {
        assert_eq!(rewrite("We utilise this.", &[]), "We utilise this.");
    }

    // ---------- match metadata ----------

    #[test]
    fn matches_carry_position_and_rule_id() {
        let rules = [lit("utilise", "utilise", "use")];
        let matches = find_literal_matches("We utilise this.", &rules);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start, 3);
        assert_eq!(matches[0].end, 10);
        assert_eq!(matches[0].rule_id, "utilise");
    }
}
