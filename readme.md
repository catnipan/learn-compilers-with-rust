# RegExp

- A simple library for parsing, compiling, and executing regular expressions in Rust.

- Provided full support for escape character, concatenation, alternation and Kleene star symbols.

- Completed both DFA(deterministic finite automaton) and NFA(non-determinisitc finite automaton) implementation.

## Usage

DFA implementation

```Rust
let regexp = DFAOne::from_regexp("(a|bc)*abb", "abc");
assert!(regexp.test("abcabb"));
assert!(regexp.test("aabb"));
assert!(!regexp.test("abcbcabcaabbc"));
assert!(!regexp.test("abcbcabbc"));
```

NFA implementation

```Rust
let num_exp = NFAOne::from_regexp("((1|2|3|4|5|6|7|8|9)(0|1|2|3|4|5|6|7|8|9)*|0)(.(0|1|2|3|4|5|6|7|8|9)+)?");
assert!(num_exp.test("0"));
assert!(num_exp.test("4"));
assert!(num_exp.test("10"));
assert!(num_exp.test("12.34"));
assert!(num_exp.test("1323423"));
assert!(!num_exp.test("01323423"));
assert!(!num_exp.test("00"));
assert!(!num_exp.test("010"));
assert!(num_exp.test("0.1"));
assert!(num_exp.test("0.01"));
assert!(!num_exp.test("0."));
assert!(num_exp.test("0.123"));
assert!(!num_exp.test("01.123"));
assert!(!num_exp.test("01."));
```