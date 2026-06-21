# derex
*derex* - short for **der**ivative reg**ex** - implements an elegant alternative solution to matching regular expressions leveraging Antimirov's [partial derivatives (1996)](https://www.sciencedirect.com/science/article/pii/0304397595001824). Classical implementations rely on finite automata usually constructed via Thompson or powerset constructions. The approach of using partial derivatives allows treating regular expressions as algebraic objects and computing "next states" directly via symbolic differentiation with respect to input symbols. Explicit automaton construction can be entirely avoided in many cases.

This method allows definitions to be clean and recursive. It relies on strong theoretical properties, such as the finiteness of the set of partial derivatives as well as the so-called "subterm property", stating that every derivative is built from subexpressions of the original regular expression. This can serve as a strong foundation for advanced regex features such as sub-matching, error correction, or even automated "fixes" for the expression in case of matching failures.

## Regular Expressions
This implementation contains a minimal representation of regular expressions. Regular expressions are constructed recursively in the following form:
Let $\Sigma$ be the input alphabet and $r,s \in \Sigma^\ast$ regular expressions.
| regex      | matches                                                                                            |
|------------|----------------------------------------------------------------------------------------------------|
| $\epsilon$ | Matches the empty string.                                                                          |
| $a$        | Matches a specific character $a \in \Sigma$.                                                       |
| $rs$       | Matches any string that is a concatenation of a string matched by $r$ and a string matched by $s$. |
| $r\|s$     | Alternation. Matches any string that is either matched by $r$ or $s$.                              |
| $r^\ast$   | Kleene star. Matches zero or more repetitions of strings matched by $r$.                           |

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

### Associativity
In addition to `Seq`, representing the concatenation of two expressions, there is also `Seqs`, representing the concatenation of $n \in \mathbb{N}, n > 1$ regular expressions. This makes use of the associative property of regular expressions:

Let $r,s,t$ be regular expressions over the alphabet $\Sigma$.

$$
(rs)t = r(st)
$$

This is used in the function `fn normalize(r: R) -> R`, which consumes a regular expression `r` and returns a functionally equal but normalized expression, where all nested `Seq`s have been transformed into `Seqs`.

### Neutrality of the empty word
An additional useful property is the neutrality of the empty word $\epsilon$ towards the concatenation operation:

$$
\forall r: \epsilon r = r \epsilon = r
$$
where $r$ is a regular expressions over $\Sigma$.

This is made use of in the two smart constructor methods `smart_seq` and `smart_seqs`.

## Partial Derivatives
**Definition**:
Let $r$ be a regular expression over $\Sigma$ and $a \in \Sigma$ a character of the input alphabet $\Sigma$. We define the partial derivative of $r$ with regard to $a$ as follows:
$$
\delta_a(r) = \\{w \mid aw \in \mathcal{L}(r)\\}
$$
Here, $\mathcal{L}(r)$ denotes the language matched by the regular expression $r$, i.e. the set of all strings over $\Sigma$ that $r$ matches.

This means that the partial derivative of $r$ represents a set of continuations of $r$ that remain after the character $a$ has been matched. Importantly, it can be shown that the set of partial word derivatives of a regular expression is always finite. To understand what a partial derivative of a regular expression does, it is best to look at some examples.

### Examples
Let $r = abb^*a$ with $a,b \in \Sigma$.

$\delta_a(r) = \\{bb^*a\\}$. After the character $a$ has been matched, any remainders that match $r$ have to be of the form $bb^*a$.

$\delta_b(r) = \\{\\}$. No words matched by $r$ can start with $b$, so the partial derivative of $r$ with regard to $b$ is the empty set.

Let $s = a^*bb^*a$ where $a, b \in \Sigma$.

$\delta_a(s) = \\{a^*bb^*a\\}$.

$\delta_b(s) = \\{bb^*a\\}$.

### Nullability
A regular expression $r$ is called nullable if it matches the empty word $\epsilon$. This is an important property in this context, because if a nullable expression is reached during the consumption of input characters, the original expression matches the input word.

Nullability can be defined recursively as well. We denote the nullability function with regard to a regular expression $r$ as $\nu(r)$.

Let $r,s$ be regular expressions and $a \in \Sigma$ be any character of the input alphabet.

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
\nu(r^*) = 1
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

### Properties of Partial Derivatives
Not only does nullability provide a criterion to determine whether a match has been found, it also allows us to define some useful properties of partial derivatives that help compute partial derivatives syntactically.

Let $r,s$ be regular expressions and $a, b \in \Sigma, a \neq b$ be any characters of the input alphabet.

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
\delta_a(rs) = \delta_a(r) \cup \nu(r) \cdot \delta_a(s)
$$
$$
\delta_a(r|s) = \delta_a(r) \cup \delta_a(s)
$$
$$
\delta_a(r^\*) = \delta_a(r) \cup r^\*
$$

These properties translate directly to the recursive function `part_deriv`, found in [derivatives.rs](src/derivatives.rs).

### Matching Regular Expressions via Partial Derivatives
Now we have everything we need to write a function that uses partial derivatives of regular expressions to determine whether a word is an element of the language described by the regular expression or not. The function looks like this:
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

After this has been done for all characters `c` in `input`, the remaining set of regular expressions in `current` is checked for nullability. If any of them are nullable, the input word matched, otherwise, it did not.
