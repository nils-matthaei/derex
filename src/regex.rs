//! A simple regular expression data type.
//!
//! This module provides a basic algebraic representation of regular expressions,
//! allowing the construction and manipulation of regex patterns programmatically.

/// A regular expression abstract syntax tree.
///
/// `R` represents a regular expression using an algebraic data type with the following constructors:
/// - `Phi`: The empty language (matches nothing)
/// - `Eps`: The empty string (matches only the empty string)
/// - `L(char)`: A single character
/// - `Seq(R, R)`: Sequential composition (concatenation)
/// - `Alt(R, R)`: Alternation (choice between two patterns)
/// - `Star(R)`: Kleene star (zero or more repetitions)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum R {
    /// The empty string. Accepts only the empty string ε.
    Eps,

    /// A single character literal.
    L(char),

    /// Sequential composition (concatenation) of two regexes.
    Seq(Box<R>, Box<R>),

    /// Sequential composition (concatenation) of n regexes.
    Seqs(Vec<R>),

    /// Alternation (choice) between two regexes.
    Alt(Box<R>, Box<R>),

    /// Kleene star. Zero or more repetitions of a regex.
    Star(Box<R>),
}

impl R {
    /// Creates a sequential composition (concatenation) of two regexes.
    ///
    /// # Arguments
    /// * `left` - The first regex to concatenate
    /// * `right` - The second regex to concatenate
    ///
    /// # Example
    /// ```
    /// # use derex::R;
    /// let ab = R::seq(R::char('a'), R::char('b')); // Matches "ab"
    /// ```
    pub fn seq(left: R, right: R) -> R {
        R::Seq(Box::new(left), Box::new(right))
    }

    /// Smart constructor for sequential composition of two regexes.
    /// Simplifies the result by eliminating redundant `Eps` terms:
    /// - `Eps . r` reduces to `r`
    /// - `r . Eps` reduces to `r`
    /// Otherwise falls back to `R::Seq`.
    pub fn smart_seq(left: R, right: R) -> R {
        match (left, right) {
            (R::Eps, right) => right,
            (left, R::Eps) => left,
            (left, right) => R::seq(left, right),
        }
    }

    /// Creates a sequential composition of multiple regexes.
    ///
    /// # Arguments
    /// * `regexes` - A vector of regexes to concatenate in order
    ///
    /// # Example
    /// ```
    /// # use derex::R;
    /// let abc = R::seqs(vec![R::char('a'), R::char('b'), R::char('c')]);
    /// ```
    pub fn seqs(regexes: Vec<R>) -> R {
        R::Seqs(regexes)
    }

    /// Smart constructor for sequential composition of two regexes.
    /// Simplifies the result by eliminating redundant `Eps` terms and
    /// flattening nested `Seqs` into a single flat `Vec`:
    /// - `Eps . r` reduces to `r`
    /// - `r . Eps` reduces to `r`
    /// - `Seqs(xs) . Seqs(ys)` flattens to `Seqs(xs ++ ys)`
    /// - `r . Seqs(ys)` prepends `r` to `ys`
    /// - `Seqs(xs) . r` appends `r` to `xs`
    /// Otherwise wraps both in a new `Seqs`.
    pub fn smart_seqs(left: R, right: R) -> R {
        match (left, right) {
            (R::Eps, right) => right,
            (left, R::Eps) => left,
            (R::Seqs(ls), R::Seqs(rs)) => R::Seqs([ls, rs].concat()),
            (left, R::Seqs(mut rs)) => {
                rs.insert(0, left);
                R::Seqs(rs)
            }
            (R::Seqs(mut ls), right) => {
                ls.push(right);
                R::Seqs(ls)
            }
            (left, right) => R::Seqs(vec![left, right]),
        }
    }

    /// Creates an alternation (choice) between two regexes.
    ///
    /// # Arguments
    /// * `left` - The first option
    /// * `right` - The second option
    ///
    /// # Example
    /// ```
    /// # use derex::R;
    /// let a\_or\_b = R::alt(R::char('a'), R::char('b')); // Matches "a" or "b"
    /// ```
    pub fn alt(left: R, right: R) -> R {
        R::Alt(Box::new(left), Box::new(right))
    }

    /// Creates a Kleene star (zero or more repetitions) of a regex.
    ///
    /// # Arguments
    /// * `inner` - The regex to repeat
    ///
    /// # Example
    /// ```
    /// # use derex::R;
    /// let a\_star = R::star(R::char('a')); // Matches "", "a", "aa", "aaa", etc.
    /// ```
    pub fn star(inner: R) -> R {
        R::Star(Box::new(inner))
    }

    /// Creates a regex matching the empty string.
    ///
    /// # Example
    /// ```
    /// # use derex::R;
    /// let empty = R::eps(); // Matches only the empty string
    /// ```
    pub fn eps() -> R {
        R::Eps
    }

    /// Creates a regex matching a single character.
    ///
    /// # Arguments
    /// * `c` - The character to match
    ///
    /// # Example
    /// ```
    /// # use derex::R;
    /// let a = R::char('a'); // Matches "a"
    /// ```
    pub fn char(c: char) -> R {
        R::L(c)
    }

    /// Creates a regular expression from a string representation.
    ///
    /// # Syntax
    /// | Pattern | Constructor    |
    /// |---------|----------------|
    /// | `a`     | `L('a')`       |
    /// | `rs`    | `Seqs([r, s])` |
    /// | `r*`    | `Star(r)`      |
    /// | `r\|s`  | `Choice(r, s)` |
    /// | `(r)`   | grouping       |
    ///
    /// # Example
    /// ```
    /// # use derex::regex::R;
    /// let r = R::from_str("(ab)*");
    /// assert!(r == R::star(R::seq(R::char('a'), R::char('b'))));
    /// ```
    pub fn from_str(s: &str) -> R {
        parse(R::Eps, s)
    }
}

/// Recursively parses a string into a regular expression,
/// accumulating the result in `prev` as it consumes one character at a time.
fn parse(prev: R, s: &str) -> R {
    match s.chars().next() {
        Some(c) => {
            let (_, rest) = s.split_at(c.len_utf8());

            match c {
                '(' => {
                    let close = find_matching_paren(rest).expect("unmatched parenthesis");
                    let (inner, after) = rest.split_at(close);
                    let after = &after[1..]; // Consume the closing parenthesis
                    parse(R::smart_seqs(prev, parse(R::Eps, inner)), after)
                }
                ')' => panic!("unmatched parenthesis"),
                '|' => R::alt(prev, parse(R::Eps, rest)),
                '*' => parse(R::star(prev), rest),
                _ => parse(R::smart_seqs(prev, R::char(c)), rest),
            }
        }
        None => prev,
    }
}

/// Finds the index of the closing parenthesis matching the opening parenthesis
/// that precedes the input string `s`.
/// Correctly handles nested parentheses by tracking depth.
/// Returns `None` if no matching closing parenthesis is found.
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Normalizes a regex into Seq-Assoc normal form.
///
/// This applies the associativity law `(r . s) . t = r . (s . t)` to eliminate
/// nested `Seq` constructors and replace them with `Seqs`. All intermediate
/// `Seqs` are flattened, so `Seqs [Seqs xs, Seqs ys]` becomes `Seqs (xs ++ ys)`.
///
/// # Arguments
/// * `r` - The regex to normalize
///
/// # Example
/// ```
/// # use derex::R;
/// let r = R::seq(R::seq(R::char('a'), R::char('b')), R::char('c'));
/// let normalized = derex::normalize(r);
/// // Now uses Seqs instead of nested Seq
/// ```
pub fn normalize(r: R) -> R {
    norm_seqs(seq_to_seqs(r))
}

/// Eliminates `Seq` constructors by converting them to `Seqs`.
///
/// This is the first step of normalization. It recursively traverses the regex
/// and replaces all `Seq` constructors with `Seqs` containing the same elements.
/// Other variants (`Alt`, `Star`) are recursively processed, while `Phi`, `Eps`,
/// and `L` are left unchanged.
///
/// # Arguments
/// * `r` - The regex to transform
fn seq_to_seqs(r: R) -> R {
    match r {
        R::Alt(left, right) => R::alt(seq_to_seqs(*left), seq_to_seqs(*right)),
        R::Star(inner) => R::star(seq_to_seqs(*inner)),
        R::Seq(left, right) => R::seqs(vec![seq_to_seqs(*left), seq_to_seqs(*right)]),
        R::Seqs(regexes) => R::seqs(regexes.into_iter().map(seq_to_seqs).collect()),
        _ => r,
    }
}

/// Flattens nested `Seqs` constructors into a single flat sequence.
///
/// This is the second step of normalization. It recursively processes the regex
/// and whenever a `Seqs` variant is encountered within another `Seqs`, it extracts
/// and flattens the inner sequence. For example, `Seqs [x, Seqs [y, z]]` becomes
/// `Seqs [x, y, z]`. Other variants (`Alt`, `Star`) are recursively processed,
/// while terminal nodes (`Phi`, `Eps`, `L`) are left unchanged.
///
/// # Arguments
/// * `r` - The regex to normalize
fn norm_seqs(r: R) -> R {
    match r {
        R::Alt(left, right) => R::alt(norm_seqs(*left), norm_seqs(*right)),
        R::Star(inner) => R::star(norm_seqs(*inner)),
        R::Seq(_, _) => unreachable!("Seq should have been removed by seq_to_seqs"),
        R::Seqs(regexes) => R::seqs(
            regexes
                .into_iter()
                .map(norm_seqs)
                .flat_map(|r| match r {
                    R::Seqs(xs) => xs,
                    r => vec![r],
                })
                .collect(),
        ),
        _ => r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Tests that normalize converts binary Seq into n-ary Seqs.
    // Seq(L('a'), L('a')) should normalize to Seqs([L('a'), L('a')]),
    // verifying that normalize eliminates all binary Seq constructors
    // in favor of the canonical n-ary Seqs representation.
    fn test_normalize_eliminates_seq() {
        let r = R::seq(R::char('a'), R::char('a'));
        let s = R::seqs(vec![R::char('a'), R::char('a')]);
        let r_norm = normalize(r);
        assert!(r_norm == s);
    }

    #[test]
    // Tests that smart_seq eliminates R::Eps on the left.
    fn test_smart_seq_eps_left() {
        let expected = R::char('a');
        assert!(R::smart_seq(R::Eps, R::char('a')) == expected);
    }

    #[test]
    // Tests that smart_seq eliminates R::Eps on the right.
    fn test_smart_seq_eps_right() {
        let expected = R::char('a');
        assert!(R::smart_seq(R::char('a'), R::Eps) == expected);
    }
    #[test]
    // Tests that smart_seq wraps two plain regexes in a new Seqs.
    // L('a') . L('b') should produce Seqs([L('a'), L('b')]).
    fn test_smart_seq_wraps_two_plain() {
        let expected = R::seq(R::char('a'), R::char('b'));
        assert!(R::smart_seq(R::char('a'), R::char('b')) == expected);
    }

    #[test]
    // Tests that smart_seqs eliminates R::Eps on the left.
    fn test_smart_seqs_eps_left() {
        let r_smart = R::smart_seqs(R::Eps, R::char('a'));
        assert!(r_smart == R::char('a'));
    }

    #[test]
    // Tests that smart_seqs eliminates R::Eps on the right.
    fn test_smart_seqs_eps_right() {
        let r_smart = R::smart_seqs(R::char('a'), R::Eps);
        assert!(r_smart == R::char('a'));
    }

    #[test]
    // Tests that smart_seqs flattens two Seqs into a single flat Seqs.
    // Seqs([L('a'), L('b')]) . Seqs([L('c'), L('d')]) should produce
    // Seqs([L('a'), L('b'), L('c'), L('d')]).
    fn test_smart_seqs_flattens_two_seqs() {
        let left = R::seqs(vec![R::char('a'), R::char('b')]);
        let right = R::seqs(vec![R::char('c'), R::char('d')]);
        let expected = R::seqs(vec![R::char('a'), R::char('b'), R::char('c'), R::char('d')]);
        assert!(R::smart_seqs(left, right) == expected);
    }

    #[test]
    // Tests that smart_seqs prepends a plain regex to an existing Seqs.
    // L('a') . Seqs([L('b'), L('c')]) should produce Seqs([L('a'), L('b'), L('c')]).
    fn test_smart_seqs_prepends_to_seqs() {
        let right = R::seqs(vec![R::char('b'), R::char('c')]);
        let expected = R::seqs(vec![R::char('a'), R::char('b'), R::char('c')]);
        assert!(R::smart_seqs(R::char('a'), right) == expected);
    }

    #[test]
    // Tests that smart_seqs appends a plain regex to an existing Seqs.
    // Seqs([L('a'), L('b')]) . L('c') should produce Seqs([L('a'), L('b'), L('c')]).
    fn test_smart_seqs_appends_to_seqs() {
        let left = R::seqs(vec![R::char('a'), R::char('b')]);
        let expected = R::seqs(vec![R::char('a'), R::char('b'), R::char('c')]);
        assert!(R::smart_seqs(left, R::char('c')) == expected);
    }

    #[test]
    // Tests that smart_seqs wraps two plain regexes in a new Seqs.
    // L('a') . L('b') should produce Seqs([L('a'), L('b')]).
    fn test_smart_seqs_wraps_two_plain() {
        let expected = R::seqs(vec![R::char('a'), R::char('b')]);
        assert!(R::smart_seqs(R::char('a'), R::char('b')) == expected);
    }

    #[test]
    fn test_parse_concat() {
        let expected = R::smart_seqs(R::L('a'), R::L('b'));
        assert_eq!(R::from_str("ab"), expected)
    }

    #[test]
    fn test_parse_alt() {
        let expected = R::alt(R::L('a'), R::L('b'));
        assert_eq!(R::from_str("a|b"), expected)
    }

    #[test]
    fn test_parse_star() {
        let expected = R::star(R::L('a'));
        assert_eq!(R::from_str("a*"), expected)
    }

    #[test]
    fn test_parse_paren() {
        let expedcted = R::alt(R::star(R::smart_seqs(R::L('a'), R::L('b'))), R::L('c'));
        assert_eq!(R::from_str("(ab)*|c"), expedcted)
    }
}
