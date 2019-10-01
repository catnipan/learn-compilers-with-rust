use super::automaton::{Automaton};

pub type NFAState = Vec<usize>;

pub struct NFAOne {
  pub states_size: usize,
  pub start: NFAState,
  pub accept: NFAState,
  pub transition_func: Box<dyn Fn(usize, Option<char>) -> NFAState>,
}

impl NFAOne {
  fn gen_state(has_state: Vec<bool>) -> NFAState {
    has_state
      .iter()
      .enumerate()
      .filter_map(|(idx, &has)| if has { Some(idx) } else { None })
      .collect()
  }

  pub fn e_closure(&self, mut state: NFAState) -> NFAState {
    let mut has_state: Vec<bool> = vec![false; self.states_size];
    for &s in &state { has_state[s] = true; }
    while let Some(curr) = state.pop() {
      for n in (self.transition_func)(curr, None) {
        if !has_state[n] {
          has_state[n] = true;
          state.push(n);
        }
      }
    }
    NFAOne::gen_state(has_state)
  }

  pub fn transition(&self, state: NFAState, chr: char) -> NFAState {
    let mut has_state: Vec<bool> = vec![false; self.states_size];
    for &s in &state {
      for n in (self.transition_func)(s, Some(chr)) {
        has_state[n] = true;
      }
    }
    NFAOne::gen_state(has_state)
  }
}

impl Automaton for NFAOne {
  fn test(&self, s: &str) -> bool {
    let mut curr_state = self.e_closure(self.start.clone());
    for chr in s.chars() {
      curr_state = self.e_closure(self.transition(curr_state, chr));
    }
    let mut has_state = vec![false; self.states_size];
    for s in curr_state { has_state[s] = true; }
    for &s in &self.accept {
      if has_state[s] {
        return true;
      }
    }
    false
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_instance_1() {
    let nfa_one = NFAOne {
      states_size: 11,
      start: vec![0],
      accept: vec![10],
      transition_func: Box::new(|s: usize, chr: Option<char>| {
          match (s, chr) {
            (0, None) => vec![1, 7],
            (1, None) => vec![2, 4],
            (2, Some('a')) => vec![3],
            (4, Some('b')) => vec![5],
            (3, None) => vec![6],
            (5, None) => vec![6],
            (6, None) => vec![1, 7],
            (7, None) => vec![0],
            (7, Some('a')) => vec![8],
            (8, Some('b')) => vec![9],
            (9, Some('b')) => vec![10],
            _ => vec![],
          }
      }),
    };

    assert!(nfa_one.test("ababb"));
    assert!(!nfa_one.test("abab"));
    assert!(nfa_one.test("abababababababb"));
    assert!(nfa_one.test("abb"));
    assert!(!nfa_one.test("ab"));
  }
}