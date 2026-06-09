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

use crate::regex::{R, normalize};

/// Checks whether a regular expression is nullable,
/// i.e. whether it accepts the empty string ε.
/// - `Eps` is nullable by definition
/// - `L` is never nullable
/// - `Seq` is nullable if both sides are nullable
/// - `Seqs` is nullable if all elements are nullable
/// - `Alt` is nullable if either side is nullable
/// - `Star` is always nullable (zero repetitions)
fn nullable(r: &R) -> bool {
    match r {
        R::Eps => true,
        R::L(_) => false,
        R::Seq(left, right) => nullable(left) && nullable(right),
        R::Seqs(rs) => rs.into_iter().all(nullable),
        R::Alt(left, right) => nullable(left) || nullable(right),
        R::Star(_) => true,
    }
}

fn part_deriv(c: char, r: &R) -> Vec<R> {
    match (c, r) {
        (_, R::Eps) => vec![],
        (x, R::L(y)) => {
            if x == *y {
                vec![R::Eps]
            } else {
                vec![]
            }
        }
        (x, R::Seq(left, right)) => {
            let mut pds: Vec<R> = part_deriv(x, left)
                .into_iter()
                .map(|left_prime| R::smart_seq(left_prime, *right.clone()))
                .collect();

            if nullable(left) {
                pds.extend(part_deriv(c, right));
            }

            pds.sort();
            pds.dedup();
            pds
        }
        (x, R::Seqs(rs)) => match rs.as_slice() {
            [] => vec![],
            [r] => part_deriv(x, r),
            [r, rest @ ..] => {
                let rest_seqs = R::seqs(rest.to_vec());
                let mut pds: Vec<R> = part_deriv(x, r)
                    .into_iter()
                    .map(|r_prime| R::smart_seqs(r_prime, rest_seqs.clone()))
                    .collect();

                if nullable(r) {
                    pds.extend(part_deriv(x, &rest_seqs));
                }

                pds.sort();
                pds.dedup();
                pds
            }
        },
        (x, R::Alt(left, right)) => {
            let mut pds: Vec<R> = [part_deriv(x, left), part_deriv(x, right)].concat();

            pds.sort();
            pds.dedup();
            pds
        }
        (x, R::Star(inner)) => {
            let star = R::star(*inner.clone());
            let mut pds: Vec<R> = part_deriv(x, inner)
                .into_iter()
                .map(|r_prime| R::smart_seq(r_prime, star.clone()))
                .collect();

            pds.sort();
            pds.dedup();
            pds
        }
    }
}

pub fn matcher(input: &str, r: &R) -> bool {
    let mut current: Vec<R> = vec![r.clone()];
    for c in input.chars() {
        current = current.iter().flat_map(|r| part_deriv(c, r)).collect();
        current.sort();
        current.dedup();
    }
    current.iter().any(nullable)
}

fn letters(r: &R) -> Vec<char> {
    match r {
        R::L(c) => vec![*c],
        R::Alt(left, right) | R::Seq(left, right) => {
            let mut ls: Vec<char> = [letters(left), letters(right)].concat();
            ls.sort();
            ls.dedup();
            ls
        }
        R::Seqs(rs) => {
            let mut ls: Vec<char> = rs.into_iter().flat_map(|s| letters(s)).collect();
            ls.sort();
            ls.dedup();
            ls
        }
        R::Star(inner) => letters(inner),
        _ => vec![],
    }
}

pub fn descendants(r: &R) -> Vec<R> {
    let alphabet = letters(r);
    let mut current = vec![r.clone()];
    let mut next: Vec<R> = vec![];
    loop {
        next = current
            .iter()
            .flat_map(|r| alphabet.iter().flat_map(|&c| part_deriv(c, r)))
            .collect();
        next.extend(current.clone());
        next.sort();
        next.dedup();
        if next == current {
            return current;
        }
        current = next;
    }
}

pub fn subterm(r: &R) -> Vec<R> {
    std::iter::once(r.clone()).chain(subterm_2(r)).collect()
}

fn subterm_2(r: &R) -> Vec<R> {
    match r {
        R::Eps | R::L(_) => vec![],
        R::Alt(left, right) | R::Seq(left, right) => [subterm(left), subterm(right)].concat(),
        R::Seqs(rs) => rs.iter().flat_map(|r| subterm(r)).collect(),
        R::Star(inner) => subterm(inner),
    }
}

pub fn prop(r: &R) -> bool {
    let rp = normalize(r.clone());
    let ds: Vec<R> = descendants(&rp)
        .into_iter()
        .map(|d| normalize(d))
        .collect();
    ds.iter().all(|d| check_shape(d, &rp))
}

fn check_shape(d: &R, r: &R) -> bool {
    match d {
        R::Eps => true,
        d if d == r => true,
        R::Seqs(ds) => ds.iter().all(|d| subterm(r).contains(d)),
        _ => unreachable!("impossible case: {:?} {:?}", r, d)
    } 
}
