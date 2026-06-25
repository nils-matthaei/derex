//! Construction and use of deterministic finite automata (DFAs) built from
//! regular expressions via the subset construction over Antimirov's partial
//! derivatives.
//!
//! This module provides:
//! - [`DFA`], representing a constructed automaton as a set of states, a
//!   transition table, a start state, and a set of final states
//! - [`DFA::new`], which builds a DFA for a given regular expression by
//!   discovering reachable sets of partial derivatives via breadth-first search
//! - [`DFA::matches`], which uses the constructed automaton to decide whether
//!   a given string is accepted, by following transitions character by character
//!   and checking whether the resulting state is final
//!
//! Unlike the recursive matcher in [`derivatives`](crate::derivatives), which
//! recomputes partial derivatives on every call, a `DFA` precomputes all
//! reachable states once so that matching becomes a simple table lookup per
//! character.

use crate::derivatives::*;
use crate::regex::*;
use std::collections::HashMap;
use std::collections::HashSet;

/// A DFA state is a deduplicated, sorted set of partial derivatives,
/// represented canonically as a `Vec<R>` so it can be used as a HashMap key.
type DfaState = Vec<R>;

/// Computes the successor DFA state reachable from `state` via character `c`.
fn step(state: &DfaState, c: char) -> DfaState {
    let mut next: Vec<R> = state.iter().flat_map(|r| part_deriv(c, r)).collect();
    next.sort();
    next.dedup();
    next
}

/// Computes whether a DfaState is an accepting state by checking if any of the
/// partial derivatives it contains are nullable.
fn is_accepting_state(state: &DfaState) -> bool {
    state.iter().any(nullable)
}

/// A deterministic finite automaton constructed from a regular expression
/// via Antimirov's partial derivatives, following the subset-construction
/// idea: each DFA state corresponds to a distinct set of partial
/// derivatives reachable from the initial expression.
pub struct DFA {
    /// Total number of states, identified by IDs `0..num_states`.
    num_states: usize,
    /// The initial state ID. This should always be 0.
    start: usize,
    /// The set of accepting (nullable) state IDs.
    accepting_states: HashSet<usize>,
    /// Transition table: (state ID, character) -> successor state ID.
    transitions: HashMap<(usize, char), usize>,
}

impl DFA {
    /// Constructs a deterministic finite automaton (DFA) for the regular expression `r`
    /// using the subset construction over Antimirov's partial derivatives.
    ///
    /// Each DFA state corresponds to a distinct, reachable set of partial derivatives
    /// of `r`, discovered via a breadth-first search starting from the singleton set
    /// `{r}`. States are identified by integer IDs in the order they are discovered,
    /// and a state is accepting if any of the regular expressions it contains is nullable.
    ///
    /// The DFA's alphabet is restricted to the characters actually appearing in `r`
    /// (see [`letters`]), since transitions on any other character would always
    /// lead to the empty (dead) state.
    pub fn new(r: &R) -> DFA {
        let alphabet = letters(r);

        let mut state_ids: HashMap<DfaState, usize> = HashMap::new();
        let mut accepting_states: HashSet<usize> = HashSet::new();
        let mut transitions: HashMap<(usize, char), usize> = HashMap::new();

        let initial: DfaState = vec![r.clone()];
        state_ids.insert(initial.clone(), 0);
        if is_accepting_state(&initial) {
            accepting_states.insert(0);
        }

        let mut frontier: Vec<DfaState> = vec![initial];

        while let Some(current) = frontier.pop() {
            let current_id: usize = state_ids[&current];
            for &c in &alphabet {
                let next = step(&current, c);
                let next_id = match state_ids.get(&next) {
                    Some(&id) => id,
                    None => {
                        let id = state_ids.len();
                        state_ids.insert(next.clone(), id);
                        if is_accepting_state(&next) {
                            accepting_states.insert(id);
                        }
                        frontier.push(next);
                        id
                    }
                };
                transitions.insert((current_id, c), next_id);
            }
        }

        DFA {
            num_states: state_ids.len(),
            start: 0,
            accepting_states,
            transitions,
        }
    }

    /// Walks the transition table character by character starting from self.start.
    /// Returns `false` immediately if a transition is missing, handling characters
    /// outside the regex's alphabet.
    ///
    /// After all characters have been consumed acceptence is checked by checking
    /// whether the final state is an accepting state of the DFA.
    pub fn matches(&self, word: &str) -> bool {
        let mut current_state = self.start;
        for c in word.chars() {
            match self.transitions.get(&(current_state, c)) {
                Some(&state) => {
                    current_state = state;
                }
                None => return false,
            }
        }
        self.accepting_states.contains(&current_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regex::{self, R, Word};
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_correct_match() {
        let r = R::from_str("ab*");
        let m = DFA::new(&r);
        assert!(m.matches("abbb"))
    }

    #[test]
    fn test_incorrect_match() {
        let r = R::from_str("ab*");
        let m = DFA::new(&r);
        assert!(!m.matches("abba"))
    }

    #[quickcheck]
    fn prop_matcher_matches_matches(r: R, w: Word) -> bool {
        let m = DFA::new(&r);
        let word = w.0.as_str();
        m.matches(word) == matcher(word, &r)
    }
}
