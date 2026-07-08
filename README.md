# derex
*derex* - short for **der**ivative reg**ex** - implements an elegant alternative solution to matching regular expressions leveraging Antimirov's [partial derivatives (1996)](https://www.sciencedirect.com/science/article/pii/0304397595001824). Classical implementations rely on finite automata usually constructed via Thompson or powerset constructions. The approach of using partial derivatives allows treating regular expressions as algebraic objects and computing "next states" directly via symbolic differentiation with respect to input symbols. Explicit automaton construction can be entirely avoided in many cases.

This method allows definitions to be clean and recursive. It relies on strong theoretical properties, such as the finiteness of the set of partial derivatives as well as the so-called "subterm property", stating that every derivative is built from subexpressions of the original regular expression. This can serve as a strong foundation for advanced regex features such as sub-matching, error correction, or even automated "fixes" for the expression in case of matching failures.

## Regular Expressions
Regular expressions are a type of language-defining notation. They are closely related to finite automata, derterministic and nondeterministic, in that they are notations for the same kind of languages: Regular languages. What makes regular expressions special is that they provide an algebraic description of regular languages with algebraic laws significantly resembling those of arithmetic algebra. Thus they offer a declarative way to express the words a language should accept. We denote the language described by a regular expression $r$ as $\mathcal{L}(r)$.
This implementation contains a minimal representation of regular expressions. Regular expressions are constructed recursively in the following form:
Let $\Sigma$ be the input alphabet and $r,s \in \Sigma^\ast$ regular expressions.

| regex      | matches                                                                                            | language 
|------------|----------------------------------------------------------------------------------------------------|-------------------------------------------------------
| $\epsilon$ | Matches the empty string.                                                                          | $\mathcal{L}(\epsilon) = \\{\epsilon\\}$
| $a$        | Matches a specific character $a \in \Sigma$.                                                       | $\mathcal{L}(a) = \\{a\\}$
| $rs$       | Matches any string that is a concatenation of a string matched by $r$ and a string matched by $s$. | $\mathcal{L}(rs) = \mathcal{L}(r)\mathcal{L}(s) = \\{wv \| w \in \mathcal{L}(r), v \in \mathcal{L}(s)\\}$
| $r\|s$     | Alternation. Matches any string that is either matched by $r$ or $s$.                              | $\mathcal{L}(r\|s) = \mathcal{L}(r)\cup\mathcal{L}(s)$
| $r^\ast$   | Kleene star. Matches zero or more repetitions of strings matched by $r$.                           | $\mathcal{L}(r^\ast) = (\mathcal{L}(r))^\ast$

To avoid parentheses it is assumed that the Kleene star has the highest priority, followed by concatenation, then alternation. 

In this implementation, this is represented in the `enum R`:
```rust
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
```
Since the size of an `R` needs to be known at compile time, it is necessary to store recursive instances of `R` in a `Box`, which are Rust's heap-allocated smart pointers.

### Examples
Let the input alphabet be $\Sigma = \\{a, b\\}$.
Over $\Sigma$ we can easily define the trivial regular expressions $r$ and $s$ containing only the single characters $a$ and $b$.

Let $r = a$, $s = b$.

$\Rightarrow \mathcal{L}(r) = \\{a\\}, \mathcal{L}(s) = \\{b\\}$

In Rust:
```rust
let r = R::L('a');
let s = R::L('b');
```

**Concatenation** $rs$:

$\mathcal{L}(rs) = \\{ab\\}$, the language containing only the single word $ab$.

```rust
let rs = R::Seq(Box::new(R::L('a')), Box::new(R::L('b')));
```

**Alternation** $r|s$:

$\mathcal{L}(r|s) = \\{a, b\\}$, the language containing all words in $\mathcal{L}(r)$ and $\mathcal{L}(s)$.

```rust
let r_or_s = R::Alt(Box::new(R::L('a')), Box::new(R::L('b')));
```

**Kleene closure** $(rs)^\ast$:

$\mathcal{L}((rs)^\ast) = \\{\epsilon, ab, abab, ababab, \ldots\\}$, the language containing the empty word and all words consisting of zero or more repetitions of $ab$.

```rust
let rs_star = R::Star(Box::new(R::Seq(Box::new(R::L('a')), Box::new(R::L('b')))));
```
### Associativity and normalization
In addition to `Seq`, representing the concatenation of two expressions, there is also `Seqs`, representing the concatenation of $n \in \mathbb{N}, n > 1$ regular expressions. This makes use of the associative property of regular expressions:

Let $r,s,t$ be regular expressions over the alphabet $\Sigma$.

$$
(rs)t = r(st)
$$

Without a canonical form, the same language could be represented by arbitrarily deep nested `Seq` trees. For example, the concatenation of three characters $a$, $b$, $c$ could be represented as either:

```rust
R::Seq(
    Box::new(R::Seq(Box::new(R::L('a')), Box::new(R::L('b')))),
    Box::new(R::L('c'))
)   // (ab)c
```
or:
```rust
R::Seq(
    Box::new(R::L('a')),
    Box::new(R::Seq(Box::new(R::L('b')), Box::new(R::L('c'))))
)   // a(bc)
```

Both represent the same language $\\{abc\\}$, but are structurally different expressions. To eliminate this ambiguity, we use the function `normalize`, which converts all nested `Seq`s into a single flat `Seqs`:

```rust
pub fn normalize(r: R) -> R {
    norm_seqs(seq_to_seqs(r))
}
```

`normalize` is composed of two passes. The first pass, `seq_to_seqs`, converts all binary `Seq` constructors into `Seqs`:

```rust
pub fn seq_to_seqs(r: R) -> R {
    match r {
        R::Alt(l, r) => R::Alt(Box::new(seq_to_seqs(*l)), Box::new(seq_to_seqs(*r))),
        R::Star(inner) => R::Star(Box::new(seq_to_seqs(*inner))),
        R::Seq(l, r) => R::Seqs(vec![seq_to_seqs(*l), seq_to_seqs(*r)]),
        R::Seqs(rs) => R::Seqs(rs.into_iter().map(seq_to_seqs).collect()),
        r => r,
    }
}
```

The second pass, `norm_seqs`, flattens any remaining nested `Seqs` into a single flat `Seqs`:

```rust
pub fn norm_seqs(r: R) -> R {
    match r {
        R::Alt(l, r) => R::Alt(Box::new(norm_seqs(*l)), Box::new(norm_seqs(*r))),
        R::Star(inner) => R::Star(Box::new(norm_seqs(*inner))),
        R::Seqs(rs) => R::Seqs(
            rs.into_iter()
                .flat_map(|r| match norm_seqs(r) {
                    R::Seqs(inner) => inner,
                    r => vec![r],
                })
                .collect(),
        ),
        r => r,
    }
}
```

After normalization, both representations of $abc$ above would be reduced to the same canonical form:

```rust
R::Seqs(vec![R::L('a'), R::L('b'), R::L('c')])
```

### Smart Constructors
Naively constructing regular expressions by directly using the enum variants can quickly lead to redundant or unnecessarily complex expression trees. For example, concatenating any expression $r$ with $\epsilon$ should yield $r$ itself, since $\epsilon$ is the identity element of concatenation:

$$
\forall r: \epsilon r = r \epsilon = r
$$

where $r$ is a regular expression over $\Sigma$.

Without accounting for this, a naive construction might produce `R::Seq(Box::new(R::Eps), Box::new(R::L('a')))` where simply `R::L('a')` would suffice. Similarly, nested `Seqs` like `Seqs([Seqs([a, b]), c])` should always be flattened to `Seqs([a, b, c])` to maintain a canonical form.

To avoid this, we make use of two **smart constructors** `smart_seq` and `smart_seqs`, which apply these simplification rules automatically during construction:

```rust
pub fn smart_seq(left: R, right: R) -> R {
    match (left, right) {
        (R::Eps, right) => right,
        (left, R::Eps) => left,
        (left, right) => R::Seq(Box::new(left), Box::new(right)),
    }
}

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
```

The match arms are ordered deliberately, so that more specific patterns (like `R::Eps` on either side) are checked before the general fallthrough case.

#### Examples 

```rust
// Eps on the left is eliminated
R::smart_seq(R::Eps, R::L('a'))    // returns R::L('a')

// Eps on the right is eliminated
R::smart_seq(R::L('a'), R::Eps)    // returns R::L('a')

// Two Seqs are flattened into one
R::smart_seqs(
    R::Seqs(vec![R::L('a'), R::L('b')]),
    R::Seqs(vec![R::L('c'), R::L('d')]),
)   // returns R::Seqs([R::L('a'), R::L('b'), R::L('c'), R::L('d')])

// A plain expression is prepended to an existing Seqs
R::smart_seqs(
    R::L('a'),
    R::Seqs(vec![R::L('b'), R::L('c')]),
)   // returns R::Seqs([R::L('a'), R::L('b'), R::L('c')])

// Two plain expressions are wrapped in a new Seqs
R::smart_seqs(R::L('a'), R::L('b'))    // returns R::Seqs([R::L('a'), R::L('b')])
```

## Partial Derivatives

### Nullability
Before defining partial derivatives, we need to introduce the concept of nullability, which plays two important roles: it appears in the inductive definition of partial derivatives for the concatenation case, and it serves as the accepting condition for the matcher.

A regular expression $r$ is called nullable if it matches the empty word $\epsilon$, i.e. $\epsilon \in \mathcal{L}(r)$. Nullability can be decided recursively as follows. We denote the nullability function for a regular expression $r$ as $\nu(r)$, where $1$ denotes true and $0$ denotes false.

Let $r, s$ be regular expressions and $a \in \Sigma$ be any character of the input alphabet.

$$
\nu(\epsilon) = 1
$$
$$
\nu(a) = 0
$$
$$
\nu(rs) = \nu(r) \wedge \nu(s)
$$
$$
\nu(r|s) = \nu(r) \vee \nu(s)
$$
$$
\nu(r^\*) = 1
$$

In code this directly translates to the following function `nullable`:
```rust
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
```

### Definition (Partial derivates)
Let $r, s$ be regular expressions over $\Sigma$ and $a, b \in \Sigma, a \neq b$ be any characters of the input alphabet. We define the partial derivative $\delta_a(r)$ of $r$ with respect to $a$ inductively by structural recursion on $r$:

$$
\delta_a(\epsilon) = \\{\\}
$$
$$
\delta_a(a) = \\{\epsilon\\}
$$
$$
\delta_a(b) = \\{\\}
$$
$$
\delta_a(rs) = \delta_a(r) \cdot s \cup \nu(r) \cdot \delta_a(s)
$$
$$
\delta_a(r|s) = \delta_a(r) \cup \delta_a(s)
$$
$$
\delta_a(r^\*) = \delta_a(r) \cdot r^\*
$$

Here $\delta_a(r) \cdot s$ denotes the set $\\{r's \mid r' \in \delta_a(r)\\}$, meaning each element of the derivative set sequenced with the regular expression $s$. The role of $\nu(r)$ in the concatenation case is to account for the possibility that $r$ itself matches the empty word, if it does, the character $a$ may also be matched by $s$, so $\delta_a(s)$ must be included as well.

It can be shown that this inductive definition satisfies the semantic property

$$
\mathcal{L}(\delta_a(r)) = \\{w \mid aw \in \mathcal{L}(r)\\}
$$

where $\mathcal{L}(\delta_a(r)) = \bigcup_{r' \in \delta_a(r)} \mathcal{L}(r')$ denotes the union of the languages of all expressions in the derivative set (Antimirov, 1996).

This means, that the partial derivatives of $r$ with respect to $a$ are exactly the continuations that remain after matching $a$ against $r$ (Antimirov, 1996). Importantly, the set of all partial derivatives of a regular expression is always finite.

These rules translate directly to the recursive function `part_deriv`, found in [derivatives.rs](src/derivatives.rs).

### Examples
Let $r = abb^\*a$ with $a, b \in \Sigma$.

$\delta_a(r) = \\{bb^\*a\\}$. After the character $a$ has been matched, any remaining input that still matches $r$ must be of the form $bb^\*a$ — the leading $a$ has already been consumed.

$\delta_b(r) = \\{\\}$. No word matched by $r$ can start with $b$, so the partial derivative of $r$ with regard to $b$ is the empty set.

Let $s = a^\*bb^\*a$ where $a, b \in \Sigma$.

$\delta_a(s) = \\{a^\*bb^\*a\\}$. The character $a$ can only be matched by the $a^\*$ prefix, which after consuming one $a$ leaves the full expression $s$ unchanged, since $a^\*$ can still match further repetitions.

$\delta_b(s) = \\{bb^\*a\\}$. The character $b$ can only be matched after the $a^\*$ prefix has matched zero repetitions, leaving $bb^\*a$.

## Matching Regular Expressions via Partial Derivatives
Using partial derivatives and nullability, we can now define a matcher function that decides whether a word is an element of $\mathcal{L}(r)$. The function looks like this:
```rust
pub fn matcher(input: &str, r: &R) -> bool {
    let mut current: Vec<R> = vec![r.clone()];
    for c in input.chars() {
        current = current.iter().flat_map(|r| part_deriv(c, r)).collect();
        current.sort();
        current.dedup();
    }
    current.iter().any(nullable)
}
```
The function takes an input word `input` and a regular expression `r`.
The variable `current` stores all remaining (sub-)expressions after each iteration of the following loop. At the beginning it contains only the original expression `r`.

The function then iterates over all characters `c` in the input word `input`. For each of these characters, the partial derivatives with regard to `c` are computed for all remaining sub-expressions in `current`. The resulting sets of regular expressions are merged, stored back in `current`, and deduplicated.

After this has been done for all characters `c` in `input`, all input has been consumed and the remaining set of regular expressions in `current` represents all possible continuations of `r` after reading `input`. By our earlier definition, a regular expression is nullable if and only if it matches the empty word $\epsilon$. So if any expression in `current` is nullable, it means the empty continuation is valid and `input` is a complete match. This is checked by `current.iter().any(nullable)`, which returns `true` if at least one expression in `current` is nullable.