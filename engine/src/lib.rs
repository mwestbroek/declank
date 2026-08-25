mod book;
mod compile;
mod inflect;
mod rules;
mod rewrite;

use regex::Regex;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn rewrite(input: &str) -> String { 
    let re = Regex::new(r"(?i)\band\b").unwrap();
    let result = re.replace_all(input, "&");

    result.to_string()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let result = rewrite("A test and");
        assert_eq!(result, "A test &");
    }

    #[test]
    fn inside_word_ignored() {
        let result = rewrite("word handles word");
        assert_eq!(result, "word handles word");
    }

    #[test]
    fn at_start_and_end() {
        let result = rewrite("and blah and");
        assert_eq!(result, "& blah &"); 
    }

    #[test]
    fn multiple_replacements() {
        let result = rewrite("word and and and");
        assert_eq!(result, "word & & &");
    }

    #[test]
    fn no_matches() {
        let result = rewrite("a normal sentence");
        assert_eq!(result, "a normal sentence");
    }

    #[test]
    fn empty_str() {
        let result = rewrite("");
        assert_eq!(result, "");
    }

    #[test]
    fn case_sensitivity() {
        let result = rewrite("AND aNd And");
        assert_eq!(result, "& & &");
    }
}

