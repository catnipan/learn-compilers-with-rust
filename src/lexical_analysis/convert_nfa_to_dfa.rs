use super::dfa::DFAOne;
use super::nfa::{NFAOne, NFAState};
use super::automaton::Automaton;

use std::collections::HashMap;

fn convert_nfa_to_dfa(nfa: NFAOne, input_set: &str) -> DFAOne {
  let mut new_state_idx: usize = 0;
  let mut new_state_map: HashMap<NFAState, usize> = HashMap::new();
  let mut is_marked = vec![];
  let mut stack = vec![];
  let mut transition_map: HashMap<(usize, char), usize> = HashMap::new();

  let start = nfa.e_closure(vec![nfa.start]);
  stack.push(start.clone());
  new_state_map.insert(start, new_state_idx);
  is_marked.push(false);
  new_state_idx += 1;
  

  while let Some(curr_state) = stack.pop() {
    let &curr_state_idx = new_state_map.get(&curr_state).unwrap();
    if is_marked[curr_state_idx] {
      continue;
    }
    is_marked[curr_state_idx] = true;
    for chr in input_set.chars() {
      let new_state = nfa.e_closure(nfa.transition(curr_state.clone(), chr));
      let new_state_idx = match new_state_map.get(&new_state) {
        Some(idx) => *idx,
        None => {
          stack.push(new_state.clone());
          new_state_map.insert(new_state, new_state_idx);
          is_marked.push(false);
          let state_idx = new_state_idx;
          new_state_idx += 1;
          state_idx
        },
      };
      transition_map.insert((curr_state_idx, chr), new_state_idx);
    }
  }

  let mut accept_states = vec![false; nfa.states_size];
  for &s in &nfa.accept { accept_states[s] = true; }
  let accept = new_state_map.into_iter().filter_map(|(ss, idx)| {
    for s in ss {
      if accept_states[s] {
        return Some(idx);
      }
    }
    None
  }).collect();

  DFAOne {
    states_size: new_state_idx,
    start: 0,
    accept,
    transition_func: Box::new(move |s: usize, chr: char| {
      *transition_map.get(&(s, chr)).expect("not implemented")
    })
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
    let converted_dfa = convert_nfa_to_dfa(nfa_one, "ab");
    assert!(converted_dfa.test("aabb"));
    assert!(!converted_dfa.test("abbb"));
    assert!(converted_dfa.test("abababaabb"));
  }
}
