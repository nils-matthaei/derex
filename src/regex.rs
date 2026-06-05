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
/// - `Choice(R, R)`: Alternation (choice between two patterns)
/// - `Star(R)`: Kleene star (zero or more repetitions)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum R {
    /// The empty language. Accepts no strings.
    Phi,

    /// The empty string. Accepts only the empty string ε.
    Eps,

    /// A single character literal.
    L(char),

    /// Sequential composition (concatenation) of two regexes.
    Seq(Box<R>, Box<R>),

    /// Sequential composition (concatenation) of n regexes.
    Seqs(Vec<R>),

    /// Alternation (choice) between two regexes.
    Choice(Box<R>, Box<R>),

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
    pub fn smart_seq(left: R, right: R) -> R{
        match (left, right) {
            (R::Eps, right) => right,
            (left, R::Eps) => left,
            (left, right) => R::seq(left, right)
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
            },
            (R::Seqs(mut ls), right) => {
                ls.push(right);
                R::Seqs(ls)
            },
            (left, right) => R::Seqs(vec![left, right])
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
    /// let a\_or\_b = R::choice(R::char('a'), R::char('b')); // Matches "a" or "b"
    /// ```
    pub fn choice(left: R, right: R) -> R {
        R::Choice(Box::new(left), Box::new(right))
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

    /// Creates the empty language (matches nothing).
    ///
    /// # Example
    /// ```
    /// # use derex::R;
    /// let nothing = R::phi(); // Matches no strings
    /// ```
    pub fn phi() -> R {
        R::Phi
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
/// Other variants (`Choice`, `Star`) are recursively processed, while `Phi`, `Eps`,
/// and `L` are left unchanged.
///
/// # Arguments
/// * `r` - The regex to transform
fn seq_to_seqs(r: R) -> R {
    match r {
        R::Choice(left, right) => R::choice(seq_to_seqs(*left), seq_to_seqs(*right)),
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
/// `Seqs [x, y, z]`. Other variants (`Choice`, `Star`) are recursively processed,
/// while terminal nodes (`Phi`, `Eps`, `L`) are left unchanged.
///
/// # Arguments
/// * `r` - The regex to normalize
fn norm_seqs(r: R) -> R {
    match r {
        R::Choice(left, right) => R::choice(norm_seqs(*left), norm_seqs(*right)),
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
