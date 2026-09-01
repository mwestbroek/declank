mod book;
mod compile;
mod inflect;
mod rewrite;
mod rules;

use serde::Serialize;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

use crate::{
    book::{Book, RuleError, SerialisableBook, deserialise_book},
    compile::{CompiledRules, compile},
    rewrite::rewrite,
};

static RULES: Mutex<Option<CompiledRules>> = Mutex::new(None);

#[derive(Serialize)]
struct RuleParseReport {
    loaded: usize,
    errors: Vec<RuleError>,
}

fn set_rules_inner(json_rules: &str) -> Result<String, String> {
    let book: SerialisableBook =
        serde_json::from_str(json_rules).map_err(|e| e.to_string())?; 
    let Book {
        rules,
        errors,
        enabled,
    } = deserialise_book(book);
    let success_count = rules.len();
    let compiled_rules = compile(rules, enabled);
    {
        let mut rules = RULES.lock().unwrap();
        *rules = Some(compiled_rules);
    }
    let report = RuleParseReport {
        loaded: success_count,
        errors,
    };
    serde_json::to_string(&report).map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn set_rules(json_rules: &str) -> Result<String, JsValue> {
    set_rules_inner(json_rules).map_err(|e| JsValue::from_str(&e))
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

    // All these tests share one global RULES, and cargo runs tests in
    // parallel. This lock serialises them. `unwrap_or_else` ignores poisoning
    // so that one failing test does not cascade into failures in the rest.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn guard() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    const GOOD_BOOK: &str = r#"{
        "version": 1,
        "rules": [
            {"id":"utilise","kind":"lemma-verb","pattern":"utilise","replacement":"use"},
            {"id":"tapestry","kind":"literal","pattern":"rich tapestry of","replacement":"range of"}
        ]
    }"#;

    #[test]
    fn set_rules_loads_a_book_and_declank_uses_it() {
        let _g = guard();
        let report = set_rules_inner(GOOD_BOOK).unwrap();
        assert!(report.contains("\"loaded\":2"), "report was {report}");
        assert_eq!(declank("We utilise this."), "We use this.");
    }

    #[test]
    fn lemma_expansion_survives_the_round_trip() {
        let _g = guard();
        set_rules_inner(GOOD_BOOK).unwrap();
        assert_eq!(declank("She utilises this."), "She uses this.");
        assert_eq!(declank("They utilised this."), "They used this.");
        assert_eq!(declank("We are utilising this."), "We are using this.");
    }

    #[test]
    fn an_empty_book_rewrites_nothing() {
        let _g = guard();
        set_rules_inner(r#"{"version":1,"rules":[]}"#).unwrap();
        assert_eq!(declank("We utilise this."), "We utilise this.");
    }

    #[test]
    fn a_disabled_book_rewrites_nothing() {
        let _g = guard();
        let json = r#"{
            "version": 1,
            "enabled": false,
            "rules": [
                {"id":"utilise","kind":"lemma-verb","pattern":"utilise","replacement":"use"}
            ]
        }"#;
        let report = set_rules_inner(json).unwrap();
        // Rules still load; the switch only gates rewriting.
        assert!(report.contains("\"loaded\":1"), "report was {report}");
        assert_eq!(declank("We utilise this."), "We utilise this.");
    }

    #[test]
    fn a_disabled_rule_is_skipped() {
        let _g = guard();
        let json = r#"{
            "version": 1,
            "rules": [
                {"id":"utilise","enabled":false,"kind":"lemma-verb","pattern":"utilise","replacement":"use"},
                {"id":"tapestry","kind":"literal","pattern":"rich tapestry of","replacement":"range of"}
            ]
        }"#;
        set_rules_inner(json).unwrap();
        assert_eq!(declank("We utilise this."), "We utilise this.");
        assert_eq!(declank("a rich tapestry of things"), "a range of things");
    }

    #[test]
    fn malformed_json_is_rejected_and_leaves_the_old_rules_in_place() {
        let _g = guard();
        set_rules_inner(GOOD_BOOK).unwrap();
        assert!(set_rules_inner("{ not json").is_err());
        assert_eq!(declank("We utilise this."), "We use this.");
    }

    #[test]
    fn a_bad_template_is_reported_and_the_rest_still_load() {
        let _g = guard();
        // "{A} starts here" begins with a hole, so it fails EmptyLead.
        let json = r#"{
            "version": 1,
            "rules": [
                {"id":"utilise","kind":"lemma-verb","pattern":"utilise","replacement":"use"},
                {"id":"broken","kind":"template","pattern":"{A} starts here","replacement":"{A}"}
            ]
        }"#;
        let report = set_rules_inner(json).unwrap();
        assert!(report.contains("\"loaded\":1"), "report was {report}");
        assert!(report.contains("broken"), "report was {report}");
        assert!(report.contains("lead is empty"), "report was {report}");
        // The good rule still works.
        assert_eq!(declank("We utilise this."), "We use this.");
    }

    #[test]
    fn a_valid_template_round_trips() {
        let _g = guard();
        let json = r#"{
            "version": 1,
            "rules": [
                {"id":"in-question","kind":"template","pattern":"the {A} in question","replacement":"the {A}"}
            ]
        }"#;
        set_rules_inner(json).unwrap();
        assert_eq!(
            declank("We reviewed the £4.2m figure in question today."),
            "We reviewed the £4.2m figure today."
        );
    }
}
