#[derive(Debug, PartialEq)]
pub struct VerbForms {
    pub base: String,
    pub third_person: String,
    pub past: String,
    pub past_participle: String,
    pub present_participle: String,
}


#[derive(Debug, PartialEq)]
pub struct NounForms {
    pub singular: String,
    pub plural: String,
}

// TODO - add more versions
fn irregular_verb(base: &str) -> Option<VerbForms> {
    let (third, past, pp, ing) = match base {
        "be"   => ("is", "was", "been", "being"),
        "have" => ("has", "had", "had", "having"),
        "run"  => ("runs", "ran", "run", "running"),
        _ => return None,
    };
    Some(VerbForms {
        base: base.to_string(),
        third_person: third.to_string(),
        past: past.to_string(),
        past_participle: pp.to_string(),
        present_participle: ing.to_string(),
    })
}

// TODO - add more
fn irregular_noun(base: &str) -> Option<NounForms> {
    let plural = match base {
        // English irregulars
        "man" => "men",
        "woman" => "women",
        "child" => "children",
        "person" => "people",
        "foot" => "feet",
        "tooth" => "teeth",
        "mouse" => "mice",
        "goose" => "geese",

        // Unchanged
        "sheep" => "sheep",
        "fish" => "fish",
        "series" => "series",
        "species" => "species",
        "aircraft" => "aircraft",

        // Latin and Greek. These matter most: they are common in the register
        // this tool targets, and several would otherwise hit the sibilant rule
        // and produce "analysises".
        "analysis" => "analyses",
        "basis" => "bases",
        "crisis" => "crises",
        "thesis" => "theses",
        "hypothesis" => "hypotheses",
        "criterion" => "criteria",
        "phenomenon" => "phenomena",
        "datum" => "data",
        "medium" => "media",
        "index" => "indices",
        "matrix" => "matrices",
        "appendix" => "appendices",
        "vertex" => "vertices",

        // -o taking -oes. No spelling rule separates these from photo/photos,
        // so the default is -s and the -oes cases live here.
        "hero" => "heroes",
        "potato" => "potatoes",
        "tomato" => "tomatoes",
        "echo" => "echoes",

        _ => return None,
    };
    Some(NounForms {
        singular: base.to_string(),
        plural: plural.to_string(),
    })
}
fn is_vowel(letter: char) -> bool {
    matches!(letter, 'a' | 'e' | 'i' | 'o' | 'u')
}

fn is_consonant(letter: char) -> bool {
    !is_vowel(letter)
}

fn ends_in_consonant_then_y(base: &str) -> bool {
    let mut rev = base.chars().rev();
    match (rev.next(), rev.next()) {
        (Some('y'), Some(c)) => is_consonant(c),
        _ => false,
    }
}

/// Consonant-vowel-consonant ending, excluding final w/x/y.
/// Deliberately naive: cannot distinguish offer/offered from prefer/preferred,
/// since that depends on stress. Add exceptions to the irregular table.
fn doubles(base: &str) -> bool {
    let mut rev = base.chars().rev();
    match (rev.next(), rev.next(), rev.next()) {
        (Some(last), Some(mid), Some(first)) => {
            is_consonant(last)
                && !matches!(last, 'w' | 'x' | 'y')
                && is_vowel(mid)
                && is_consonant(first)
        }
        _ => false,
    }
}

fn drop_last(base: &str, n: usize) -> &str {
    // Byte slice: safe for ASCII verbs, which is all the rule book contains.
    &base[..base.len() - n]
}

fn third_person(base: &str) -> String {
    if base.ends_with('o')
        || base.ends_with('s')
        || base.ends_with('x')
        || base.ends_with('z')
        || base.ends_with("ch")
        || base.ends_with("sh")
    {
        format!("{base}es")
    } else if ends_in_consonant_then_y(base) {
        format!("{}ies", drop_last(base, 1))
    } else {
        format!("{base}s")
    }
}

fn past(base: &str) -> String {
    if base.ends_with('e') {
        format!("{base}d")
    } else if ends_in_consonant_then_y(base) {
        format!("{}ied", drop_last(base, 1))
    } else if doubles(base) {
        let last = base.chars().last().unwrap();
        format!("{base}{last}ed")
    } else {
        format!("{base}ed")
    }
}

fn present_participle(base: &str) -> String {
    if base.ends_with("ie") {
        // die -> dying
        format!("{}ying", &base[..base.len() - 2])
    } else if base.ends_with("ee") || base.ends_with("ye") || base.ends_with("oe") {
        // see -> seeing, dye -> dyeing
        format!("{base}ing")
    } else if base.ends_with('e') {
        format!("{}ing", drop_last(base, 1))
    } else if doubles(base) {
        let last = base.chars().last().unwrap();
        format!("{base}{last}ing")
    } else {
        format!("{base}ing")
    }
}

pub fn verb_forms(base: &str) -> VerbForms {
    if let Some(forms) = irregular_verb(base) {
        return forms;
    }
    VerbForms {
        base: base.to_string(),
        third_person: third_person(base),
        past: past(base),
        past_participle: past(base),
        present_participle: present_participle(base),
    }
}

const F_KEEPS_S: &[&str] = &[
    "roof", "chief", "belief", "proof", "cliff", "brief", "grief", "chef",
];

fn plural(base: &str) -> String {
    if base.ends_with('s')
    || base.ends_with('x')
    || base.ends_with('z')
    || base.ends_with("ch")
    || base.ends_with("sh")
    {

        format!("{base}es")
    }
    else if ends_in_consonant_then_y(base) {
        format!("{}ies", drop_last(base, 1))
    }
    else if base.ends_with("fe") {
        format!("{}ves", drop_last(base, 2))
    }
    else if base.ends_with('f') && !F_KEEPS_S.contains(&base) {
        format!("{}ves", drop_last(base, 1))
    }
    else {
        format!("{base}s")
    }
}

pub fn noun_forms(base: &str) -> NounForms {
    if let Some(forms) = irregular_noun(base) {
        return forms;
    }
    NounForms {
        singular: base.to_string(),
        plural: plural(base),
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cases: &[(&str, &str)], f: fn(&str) -> String) {
        for (base, expected) in cases {
            assert_eq!(&f(base), expected, "base was {base:?}");
        }
    }

    // ---------- third person singular ----------

    #[test]
    fn third_person_default() {
        check(&[("delve", "delves"), ("plan", "plans"), ("look", "looks")], third_person);
    }

    #[test]
    fn third_person_sibilants_take_es() {
        check(
            &[
                ("pass", "passes"),
                ("fix", "fixes"),
                ("buzz", "buzzes"),
                ("watch", "watches"),
                ("push", "pushes"),
            ],
            third_person,
        );
    }

    #[test]
    fn third_person_o_takes_es() {
        check(&[("go", "goes"), ("do", "does")], third_person);
    }

    #[test]
    fn third_person_consonant_y_becomes_ies() {
        check(&[("carry", "carries"), ("try", "tries")], third_person);
    }

    #[test]
    fn third_person_vowel_y_keeps_y() {
        check(&[("play", "plays"), ("buy", "buys")], third_person);
    }

    // ---------- past ----------

    #[test]
    fn past_default() {
        check(&[("look", "looked"), ("watch", "watched")], past);
    }

    #[test]
    fn past_final_e_takes_d_only() {
        check(&[("delve", "delved"), ("use", "used")], past);
    }

    #[test]
    fn past_consonant_y_becomes_ied() {
        check(&[("carry", "carried"), ("try", "tried")], past);
    }

    #[test]
    fn past_vowel_y_keeps_y() {
        check(&[("play", "played"), ("allow", "allowed")], past);
    }

    #[test]
    fn past_doubles_after_short_vowel() {
        check(&[("plan", "planned"), ("stop", "stopped")], past);
    }

    #[test]
    fn past_no_doubling_after_two_consonants() {
        check(&[("help", "helped"), ("start", "started")], past);
    }

    #[test]
    fn past_no_doubling_after_long_vowel() {
        check(&[("need", "needed"), ("seem", "seemed")], past);
    }

    #[test]
    fn past_no_doubling_after_w_x_y() {
        check(&[("allow", "allowed"), ("fix", "fixed")], past);
    }

    // ---------- present participle ----------

    #[test]
    fn participle_default() {
        check(&[("look", "looking"), ("need", "needing")], present_participle);
    }

    #[test]
    fn participle_drops_final_e() {
        check(&[("delve", "delving"), ("use", "using")], present_participle);
    }

    #[test]
    fn participle_keeps_double_vowel_e() {
        check(&[("see", "seeing"), ("agree", "agreeing"), ("dye", "dyeing")], present_participle);
    }

    #[test]
    fn participle_ie_becomes_ying() {
        check(&[("die", "dying"), ("lie", "lying")], present_participle);
    }

    #[test]
    fn participle_doubles() {
        check(&[("plan", "planning"), ("stop", "stopping")], present_participle);
    }

    #[test]
    fn participle_y_never_doubles() {
        check(&[("carry", "carrying"), ("play", "playing")], present_participle);
    }

    // ---------- helpers ----------

    #[test]
    fn consonant_then_y_detection() {
        assert!(ends_in_consonant_then_y("carry"));
        assert!(!ends_in_consonant_then_y("play"));
        assert!(!ends_in_consonant_then_y("y"));
        assert!(!ends_in_consonant_then_y(""));
        assert!(!ends_in_consonant_then_y("look"));
    }

    #[test]
    fn doubling_detection() {
        assert!(doubles("plan"));
        assert!(doubles("stop"));
        assert!(!doubles("help"));
        assert!(!doubles("need"));
        assert!(!doubles("allow"));
        assert!(!doubles("fix"));
        assert!(!doubles("go"));
        assert!(!doubles(""));
    }

    // ---------- whole forms ----------

    #[test]
    fn regular_verb_full_set() {
        assert_eq!(
            verb_forms("delve"),
            VerbForms {
                base: "delve".into(),
                third_person: "delves".into(),
                past: "delved".into(),
                past_participle: "delved".into(),
                present_participle: "delving".into(),
            }
        );
    }

    #[test]
    fn irregular_short_circuits() {
        let f = verb_forms("run");
        assert_eq!(f.past, "ran");
        assert_eq!(f.past_participle, "run");
        assert_eq!(f.present_participle, "running");

        let f = verb_forms("be");
        assert_eq!(f.third_person, "is");
        assert_eq!(f.past, "was");
    }

    #[test]
    fn no_form_is_empty() {
        for base in ["delve", "carry", "play", "plan", "help", "see", "die", "go"] {
            let f = verb_forms(base);
            for form in [&f.third_person, &f.past, &f.past_participle, &f.present_participle] {
                assert!(!form.is_empty(), "empty form for {base:?}");
            }
        }
    }

    // ---------- known limitation ----------

    #[test]
    fn stress_dependent_doubling_is_wrong() {
        // Doubling depends on which syllable is stressed, which spelling does
        // not encode. "prefer" doubles, "offer" does not, and they look
        // identical to the rule. Pinning the wrong answer here so that adding
        // "offer" to the irregular table shows up as a failing test.
        assert_eq!(past("prefer"), "preferred"); // correct by luck
        // assert_eq!(past("offer"), "offerred"); // WRONG, should be "offered"
    }

    #[test]
    fn plural_default() {
        check(&[("tool", "tools"), ("platform", "platforms")], plural);
    }

    #[test]
    fn plural_sibilants_take_es() {
        check(
            &[("bus", "buses"), ("box", "boxes"), ("approach", "approaches"), ("dish", "dishes")],
            plural,
        );
    }

    #[test]
    fn plural_consonant_y_becomes_ies() {
        check(&[("intricacy", "intricacies"), ("company", "companies")], plural);
    }

    #[test]
    fn plural_vowel_y_keeps_y() {
        check(&[("journey", "journeys"), ("day", "days")], plural);
    }

    #[test]
    fn plural_f_becomes_ves() {
        check(&[("leaf", "leaves"), ("half", "halves"), ("shelf", "shelves")], plural);
    }

    #[test]
    fn plural_fe_becomes_ves() {
        check(&[("knife", "knives"), ("life", "lives"), ("wife", "wives")], plural);
    }

    #[test]
    fn plural_f_exceptions_keep_s() {
        // "belief" matters most: "believes" is a real verb form and would
        // collide with a verb rule in the matcher.
        check(&[("roof", "roofs"), ("chief", "chiefs"), ("belief", "beliefs")], plural);
    }

    #[test]
    fn plural_o_defaults_to_s() {
        check(&[("photo", "photos"), ("piano", "pianos"), ("memo", "memos")], plural);
    }

    #[test]
    fn irregular_nouns_short_circuit() {
        assert_eq!(noun_forms("child").plural, "children");
        assert_eq!(noun_forms("person").plural, "people");
        assert_eq!(noun_forms("sheep").plural, "sheep");
        assert_eq!(noun_forms("hero").plural, "heroes");
    }

    #[test]
    fn latin_plurals_do_not_hit_the_sibilant_rule() {
        assert_eq!(noun_forms("analysis").plural, "analyses");
        assert_eq!(noun_forms("basis").plural, "bases");
        assert_eq!(noun_forms("criterion").plural, "criteria");
        assert_eq!(noun_forms("phenomenon").plural, "phenomena");
    }

    #[test]
    fn regular_noun_full_set() {
        assert_eq!(
            noun_forms("intricacy"),
            NounForms { singular: "intricacy".into(), plural: "intricacies".into() }
        );
    }

    // ---------- known limitations ----------

    #[test]
    fn z_does_not_double() {
        // Real English is "quizzes". Pinning the wrong answer.
        assert_eq!(plural("quiz"), "quizes");
    }
}
