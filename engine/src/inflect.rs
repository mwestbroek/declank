pub struct VerbForms {
    pub base: String,
    pub third_person: String,
    pub past: String,
    pub past_participle: String,
    pub present_participle: String,
}

// TODO - add more versions
fn irregular(base: &str) -> Option<VerbForms> {
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

fn drop_last(base: &str) -> &str {
    // Byte slice: safe for ASCII verbs, which is all the rule book contains.
    &base[..base.len() - 1]
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
        format!("{}ies", drop_last(base))
    } else {
        format!("{base}s")
    }
}

fn past(base: &str) -> String {
    if base.ends_with('e') {
        format!("{base}d")
    } else if ends_in_consonant_then_y(base) {
        format!("{}ied", drop_last(base))
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
        format!("{}ing", drop_last(base))
    } else if doubles(base) {
        let last = base.chars().last().unwrap();
        format!("{base}{last}ing")
    } else {
        format!("{base}ing")
    }
}

pub fn verb_forms(base: &str) -> VerbForms {
    if let Some(forms) = irregular(base) {
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
