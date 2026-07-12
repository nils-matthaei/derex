# Implementing Regular Expression Matching via Partial Derivatives in Rust


## Background and Motivation

  

Regular expressions (regex) are a fundamental tool in computer science and software engineering,

used for pattern matching in text processing, compilers, editors, and search tools.

Classic implementations rely on finite automata (NFA/DFA) constructed via Thompson or powerset constructions.

An elegant alternative uses derivatives (Brzozowski, 1964) or [partial derivatives (Antimirov, 1996)](https://www.sciencedirect.com/science/article/pii/0304397595001824) of regular expressions.

These approaches treat regex as algebraic objects and compute "next states" directly by symbolic differentiation

with respect to input symbols, avoiding explicit automaton construction in many cases.

  

This method supports clean, recursive definitions and has strong theoretical properties, such as the

finiteness of the set of (partial) derivatives and the subterm property (every derivative is built from subexpressions of

the original regex). It is particularly insightful for understanding regex semantics and can serve as a foundation

for advanced features like sub-matching, error correction, or "fixing" matching failures.

  

## Objectives

  

The primary goal of this thesis is to develop a solid understanding of regex semantics via partial derivatives and

to produce a correct, efficient implementation in Rust. Specifically:

  

* Study the theoretical foundations of regular expressions and their partial derivatives.

  

* Re-implement a given Haskell prototype (normalization, nullable check, partial derivative computation, and a basic matcher) in Rust.

  

* Ensure the implementation is idiomatic, well-tested, and documented.

  

* Evaluate correctness and basic performance on standard regex benchmarks.

  

If time permits, explore extensions such as pretty-printing, visualization of derivative sets, or initial steps

toward handling matching failures (building on prior unpublished work by the supervisor).

  

## Methodology

  

The project proceeds in clear phases:

  

* Literature study and familiarization with partial derivatives (using provided lecture notes and selected papers).

  

* Design and implementation of the core data structures and algorithms in Rust.

  

* Testing against the Haskell reference and standard examples.

  

* Documentation and optional extensions.

  

Rust is chosen for its performance focus, strong type system, and ownership model, which aids in managing expression trees safely.

  

## Expected Outcomes and Deliverables

  

* A well-documented Rust library/crate implementing the matcher.

  

* A written thesis (≈ 30–40 pages) covering theory, implementation details, evaluation, and lessons learned.

  

* Test suite and usage examples.

  

* (Optional) A short report or presentation on insights for advanced "fixing" techniques.

  

This work contributes a modern, performant implementation of an elegant matching algorithm and prepares the ground for potential future research on regex debugging and repair.

  

## Details

  

### References

  

The paper on [partial derivatives (Antimirov, 1996)](https://www.sciencedirect.com/science/article/pii/0304397595001824)

  

Some [lecture notes on partial derivatives](https://sulzmann.github.io/ProgrammingParadigms/pp-regular-expressions.html)

  

Some pretty good [blog post](https://semantic-domain.blogspot.com/2013/11/antimirov-derivatives-for-regular.html)

   

Search for "regular expressions partial deriatives".

  

### Work steps

  

1. Study partial derivatives

  

2. Start implementing in Rust (e.g., start with a enum representation of regular expressions, nullability check, ...)

  

3. Meet regularly

  
  
  
  

### Haskell prototype

  

~~~~{.haskell}

  
  

{-# LANGUAGE GADTs #-}

  
  
  

import Data.List

import Data.Maybe

  

import Control.Monad.State

  

-- There are two ways to describe a sequence of expressions.

-- Either via Seq or Seqs.

-- Seq represents a binary operator whereas Seqs is an n-ary operator.

data RE where

Eps :: RE

L :: Char -> RE

Seq :: RE -> RE -> RE

Seqs :: [RE] -> RE

Alt :: RE -> RE -> RE

Star :: RE -> RE

deriving (Ord, Eq, Show)

  
  

-- Put into a Seq-Assoc normal form where

-- we apply the law that (r . s) . t = r . (s . t).

-- In terms of our Haskell representation,

-- we get rid of Seq and only use Seqs.

normalize :: RE -> RE

normalize = normSeqs . seqToSeqs

-- We could do both steps in one pass.

  

-- Get rid of Seq

seqToSeqs :: RE -> RE

seqToSeqs (Alt r s) = Alt (seqToSeqs r) (seqToSeqs s)

seqToSeqs (Star r) = Star $ seqToSeqs r

seqToSeqs (Seq r s) = Seqs [seqToSeqs r, seqToSeqs s]

seqToSeqs (Seqs rs) = Seqs $ map seqToSeqs rs

seqToSeqs r = r

  

-- Remove intermediate Seqs, e.g.

-- Seqs [Seqs xs, Seqs ys] => Seqs (xs ++ ys)

normSeqs :: RE -> RE

normSeqs (Alt r s) = Alt (normSeqs r) (normSeqs s)

normSeqs (Star r) = Star $ normSeqs r

normSeqs (Seq r s) = error "impossible, must have been removed"

normSeqs (Seqs rs) =

Seqs $ concat $

map unSeqs $

map normSeqs rs

where

unSeqs (Seqs xs) = xs

unSeqs r = [r]

normSeqs r = r

  
  

-- Partial derivatives

  

nullable :: RE -> Bool

nullable Eps{} = True

nullable L{} = False

nullable (Seq r s) = nullable r && nullable s

nullable (Seqs rs) = all nullable rs

nullable (Alt r s) = nullable r || nullable s

nullable Star{} = True

  

smartSeq Eps{} r2 = r2

smartSeq r1 Eps{} = r1

smartSeq r1 r2 = Seq r1 r2

  

smartSeqs Eps{} r = r

smartSeqs r Eps{} = r

smartSeqs (Seqs xs) (Seqs ys) = Seqs $ xs ++ ys

smartSeqs r (Seqs ys) = Seqs $ r : ys

smartSeqs (Seqs xs) r = Seqs $ xs ++ [r]

smartSeqs r s = Seqs [r,s]

  

partDeriv :: Char -> RE -> [RE]

partDeriv x (Eps) = []

partDeriv x (L y)

| x == y = [Eps]

| otherwise = []

partDeriv x (Seq r1 r2)

| nullable r1 = nub $ [ smartSeq r1' r2 | r1' <- partDeriv x r1 ]

++ partDeriv x r2

| otherwise = [ smartSeq r1' r2 | r1' <- partDeriv x r1 ]

  

-- The first two cases are necessary, because of the recursive call PD-Seqs

partDeriv x (Seqs []) = []

partDeriv x (Seqs [r]) = partDeriv x r

partDeriv x (Seqs (r:rs))

| nullable r = nub $ [ smartSeqs r' (Seqs rs) | r' <- partDeriv x r ]

++ partDeriv x (Seqs rs) -- PD-Seqs

| otherwise = [ smartSeqs r' (Seqs rs) | r' <- partDeriv x r ]

partDeriv x (Alt r1 r2) =

nub $ partDeriv x r1 ++ partDeriv x r2

partDeriv x (Star r) = [ smartSeq r' (Star r)| r' <- partDeriv x r ]

  

matcher :: [Char] -> RE -> Bool

matcher xs r =

go [r] xs

where

go rs [] = any nullable rs

go rs (x:xs) = go (nub $ concat [partDeriv x r | r <- rs]) xs

  
  

-- Computes all descendants of some expression r.

-- Example:

-- (ab+a)*

-- has the descendants

-- { (ab+a)*, b(ab+a)* }

--

-- We write ab as a shorthand for (a.b).

-- * binds tighter than + and .

descendants r = go [r]

where

letters :: RE -> [Char]

letters (L c) = [c]

letters (Alt r s) = nub $ (letters r) ++ (letters s)

letters (Seq r s) = nub $ (letters r) ++ (letters s)

letters (Seqs rs) = nub $ concat $ map letters rs

letters (Star r) = letters r

letters _ = []

  

alphabet = letters r

go curr =

let next = sort $ nub

$ curr

++

concat [ partDeriv x r | r <- curr, x <- alphabet ]

in if next == curr then curr

else go next

  

-- Exactracts all subterms including the expression itself.

-- NOTE.

-- There may be duplicates. For example,

-- subTerm applied to Seq (L 'a') (L 'a') yields

-- [Seq (L 'a') (L 'a'),L 'a',L 'a']

-- We could eliminate one of the occurences of L 'a'

subTerm :: RE -> [RE]

subTerm r = r : subTerm2 r

  

subTerm2 :: RE -> [RE]

subTerm2 Eps{} = []

subTerm2 L{} = []

subTerm2 (Seq r s) = subTerm r ++ subTerm s

subTerm2 (Seqs rs) = concat $ map subTerm rs

subTerm2 (Alt r s) = subTerm r ++ subTerm s

subTerm2 (Star r) = subTerm r

  
  

-- Property:

-- Descendants can be represented as subterms of the original expression.

-- Result by Antimirov:

-- For each descendent d of r we have that:

-- Either

-- (1) d equals Eps, or

-- (2) d equals r, or

-- (3) d is a sequence [x1,...,xn] where xi arer subterms of r

prop r' =

-- Normalize all expressions.

-- Then (r . (s . t)) becomes [r,s,t] from which we extract the subterms {[r,s,t],r,s,t}.

-- This makes it easier to check for the Antimirov-Subterm Property.

let r = normalize r'

ds = map normalize $ descendants r

checkShape d

| d == Eps = True

| d == r = True

| otherwise = case d of

Seqs xs -> all (\x -> elem x (subTerm r)) xs

_ -> error $ unlines ["impossible", show r, show d]

in all checkShape ds

  
  

-----------------------------------

-- Examples

  

eps = Eps

ch c = L c

conc r s = Seq r s

cons rs = Seqs rs

alt r s = Alt r s

star r = Star r

  

a = ch 'a'

b = ch 'b'

c = ch 'c'

  

ex0 = conc a a

  

ex1 = star (alt (conc a b) a)

  

ex2 = alt a (alt b c)

ex3 = alt (alt a b) c

  

ex4 = conc (conc a ex3) ex2

~~~~~~~
