//! An implementation of partial derivatives of regular expressions.
//!
//! This module provides the core building blocks of Antimirov's partial derivative algorithm:
//! - Nullability checking ([`nullable`])
//! - Partial derivative computation ([`part_deriv`])
//! - A matcher ([`matcher`]) that ties the two together
//!
//! Matching is performed by iteratively computing the set of partial derivatives
//! for each input character, and checking nullability of the resulting set
//! after all input is consumed.
//!
//! ## References
//! - Antimirov, V. (1996). Partial derivatives of regular expressions and finite automaton constructions.
//!   <https://www.sciencedirect.com/science/article/pii/0304397595001824>

use crate::regex::R;

/// Checks whether a regular expression is nullable,
/// i.e. whether it accepts the empty string ε.
/// - `Eps` is nullable by definition
/// - `L` is never nullable
/// - `Seq` is nullable if both sides are nullable
/// - `Seqs` is nullable if all elements are nullable
/// - `Choice` is nullable if either side is nullable
/// - `Star` is always nullable (zero repetitions)
fn nullable(r: R) -> bool {
    match r {
        R::Eps => true,
        R::L(_) => false,
        R::Seq(left, right) => nullable(*left) && nullable(*right),
        R::Seqs(rs) => rs.into_iter().all(nullable),
        R::Choice(left, right) => nullable(*left) || nullable(*right),
        R::Star(_) => true,
        R::Phi => false,
    }
}
