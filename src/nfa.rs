use super::automaton::Automaton;

pub type NFAState = Vec<usize>;

pub struct NFAOne {
  pub states_size: usize,
  pub start: usize,
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

  fn is_state_accept(&self, state: &NFAState) -> bool {
    let mut has_state = vec![false; self.states_size];
    for &s in state { has_state[s] = true; }
    for &s in &self.accept {
      if has_state[s] {
        return true;
      }
    }
    false
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

  pub fn transition(&self, state: &NFAState, chr: char) -> NFAState {
    let mut has_state: Vec<bool> = vec![false; self.states_size];
    for &s in state {
      for n in (self.transition_func)(s, Some(chr)) {
        has_state[n] = true;
      }
    }
    NFAOne::gen_state(has_state)
  }

  #[allow(dead_code)]
  fn simulate_by_converting_to_dfa(&self, s: &str) -> bool {
    let mut curr_state = self.e_closure(vec![self.start]);
    for chr in s.chars() {
      curr_state = self.e_closure(self.transition(&curr_state, chr));
    }
    self.is_state_accept(&curr_state)
  }

  fn simulate_on_the_fly(&self, s: &str) -> bool {
    let mut curr_stack = vec![];
    let mut next_stack = vec![];
    let mut already_on = vec![false; self.states_size];
    fn add_state(
      s: usize,
      next_stack: &mut Vec<usize>,
      already_on: &mut Vec<bool>,
      trans_func: &Box<dyn Fn(usize, Option<char>) -> NFAState>,
    ) {
      next_stack.push(s);
      already_on[s] = true;
      // add state and also calculate e-closure
      for t in trans_func(s, None) {
        if !already_on[t] {
          add_state(t, next_stack, already_on, trans_func);
        }
      }
    }

    // init e-closure of s0
    add_state(self.start, &mut curr_stack, &mut already_on, &self.transition_func);
    for &s in &curr_stack { already_on[s] = false; } // reset already_on

    for chr in s.chars() {
      
      for state in curr_stack {
        for t in (self.transition_func)(state, Some(chr)) {
          if !already_on[t] {
            next_stack.push(t);
            add_state(t, &mut next_stack, &mut already_on, &self.transition_func);
          }
        }
      }
      curr_stack = next_stack;
      next_stack = vec![];
      for &s in &curr_stack {
        already_on[s] = false; // reset
      }
    }
    self.is_state_accept(&curr_stack)
  }
}

impl Automaton for NFAOne {
  type State = Vec<usize>;

  fn init_state(&self) -> Self::State {
    self.e_closure(vec![self.start])
  }
  fn is_dead(&self, s: &Self::State) -> bool {
    s.is_empty()
  }
  fn is_accept(&self, s: &Self::State) -> bool {
    self.is_state_accept(s)
  }
  fn transition(&self, s: &Self::State, chr: char) -> Self::State {
    self.transition(s, chr)
  }

  fn test(&self, s: &str) -> bool {
    // self.simulate_by_converting_to_dfa(s)
    self.simulate_on_the_fly(s)
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_instance_1() {
    let nfa_one = NFAOne {
      states_size: 11,
      start: 0,
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