mod book;
mod compile;
mod inflect;
mod rules;
mod rewrite;

use serde::{Serialize};
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

use crate::{book::{Book, RuleError, SerialisableBook, deserialise_book}, compile::{CompiledRules, compile}, rewrite::rewrite};

static RULES: Mutex<Option<CompiledRules>> = Mutex::new(None);


#[derive(Serialize)]
struct RuleParseReport {
    loaded: usize,
    errors: Vec<RuleError>
}


#[wasm_bindgen]
pub fn set_rules(json_rules: &str) -> Result<String, JsValue> {
    let book: SerialisableBook = serde_json::from_str(json_rules).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let Book { rules, errors, enabled } = deserialise_book(book);
    let success_count = rules.len();
    let compiled_rules = compile(rules, enabled);
    {
        let mut rules = RULES.lock().unwrap();
        *rules = Some(compiled_rules);
    }
    let report = RuleParseReport { loaded: success_count, errors };
    serde_json::to_string(&report).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn declank(input: &str) -> String {
    let rules = RULES.lock().unwrap();
    match rules.as_ref() {
        Some(compiled) if compiled.enabled => rewrite(input, compiled),
        _ => input.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- lemma expansion reaches the engine ----------

    #[test]
    fn lemma_base_form() {
        assert_eq!(declank("We utilise this."), "We use this.");
    }

    #[test]
    fn lemma_third_person() {
        assert_eq!(declank("She utilises this."), "She uses this.");
    }

    #[test]
    fn lemma_past() {
        assert_eq!(declank("They utilised this."), "They used this.");
    }

    #[test]
    fn lemma_present_participle() {
        assert_eq!(declank("We are utilising this."), "We are using this.");
    }

    #[test]
    fn lemma_e_drop_in_participle() {
        // delve -> delving, not delveing; look -> looking
        assert_eq!(declank("We are delving here."), "We are looking here.");
    }

    #[test]
    fn noun_lemma_singular_and_plural() {
        assert_eq!(declank("The intricacy is clear."), "The detail is clear.");
        assert_eq!(declank("The intricacies are clear."), "The details are clear.");
    }

    // ---------- case handling ----------

    #[test]
    fn sentence_initial_capital_is_preserved() {
        assert_eq!(declank("Utilise this."), "Use this.");
    }

    #[test]
    fn all_caps_deliberately_does_not_match() {
        // Only two variants are compiled: as authored, and sentence case.
        assert_eq!(declank("UTILISE this."), "UTILISE this.");
    }

    #[test]
    fn capitalised_inflected_form_matches() {
        assert_eq!(declank("Utilising this works."), "Using this works.");
    }

    // ---------- multi-word literals ----------

    #[test]
    fn multi_word_literal() {
        assert_eq!(
            declank("We know in today's fast-paced world things move."),
            "We know currently things move."
        );
    }

    #[test]
    fn multi_word_literal_sentence_initial() {
        assert_eq!(
            declank("In today's fast-paced world things move."),
            "Currently things move."
        );
    }

    #[test]
    fn deletion_leaves_whitespace_for_the_tidy_step() {
        // No tidy step yet, so the double space is expected. This test should
        // change when whitespace normalisation lands.
        assert_eq!(
            declank("We know it is crucial to note that the system fails."),
            "We know  the system fails."
        );
    }

    // ---------- word boundaries ----------

    #[test]
    fn does_not_match_inside_a_longer_word() {
        // "delve" must not fire inside "delved" via a partial match, and
        // unrelated words containing rule text stay intact.
        assert_eq!(declank("The showcasing was good."), "The showing was good.");
        assert_eq!(declank("Underutilise this."), "Underutilise this.");
    }

    #[test]
    fn punctuation_counts_as_a_boundary() {
        assert_eq!(declank("(utilise)"), "(use)");
        assert_eq!(declank("utilise, then stop."), "use, then stop.");
    }

    // ---------- several matches at once ----------

    #[test]
    fn two_rules_in_one_sentence() {
        assert_eq!(
            declank("We utilise this to delve deeper."),
            "We use this to look deeper."
        );
    }

    #[test]
    fn distinct_rules_can_share_a_replacement() {
        // utilise and leverage both map to "use".
        assert_eq!(
            declank("We utilise and leverage it."),
            "We use and use it."
        );
    }

    #[test]
    fn same_rule_fires_repeatedly() {
        assert_eq!(
            declank("They utilise it, we utilise it."),
            "They use it, we use it."
        );
    }

    // ---------- degenerate input ----------

    #[test]
    fn no_matches_returns_input_unchanged() {
        let s = "A perfectly ordinary sentence.";
        assert_eq!(declank(s), s);
    }

    #[test]
    fn empty_input() {
        assert_eq!(declank(""), "");
    }

    #[test]
    fn whitespace_only_input() {
        assert_eq!(declank("   "), "   ");
    }

    #[test]
    fn non_ascii_input_does_not_panic() {
        // Byte-offset slicing in the matcher must not split a character.
        let s = "Wir müssen das naïve café 日本語 prüfen.";
        assert_eq!(declank(s), s);
    }

    #[test]
    fn non_ascii_around_a_match() {
        assert_eq!(declank("Wir utilise café."), "Wir use café.");
    }

    // ---------- compilation happens once ----------

    #[test]
    fn repeated_calls_are_consistent() {
        for _ in 0..3 {
            assert_eq!(declank("We utilise this."), "We use this.");
        }
    }
}