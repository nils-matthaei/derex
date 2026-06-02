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
