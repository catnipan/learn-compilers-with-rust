use super::nfa::*;
use std::collections::HashMap;
use super::automaton::Automaton;
use super::regop::RegOp;
use super::escape_chars::{EscapeChars, MaybeEsc};

struct NFABasic {
  start: usize,
  accept: usize,
}

type TransitionMapType = HashMap<(usize, Option<char>), Vec<usize>>;

struct NFAConstructor {
  state_idx: usize,
  transition_map: TransitionMapType,
}

impl NFAConstructor {
  fn new() -> Self {
    NFAConstructor {
      state_idx: 0,
      transition_map: HashMap::new(),
    }
  }

  fn gen_new_state_idx(&mut self) -> usize {
    let res = self.state_idx;
    self.state_idx += 1;
    res
  }

  fn add_new_transition(&mut self, from: usize, by: Option<char>, to: usize) {
    self.transition_map.entry((from, by)).or_insert(vec![]).push(to);
  }

  fn construct_singleton(&mut self, input: Option<char>) -> NFABasic {
    let start = self.gen_new_state_idx();
    let accept = self.gen_new_state_idx();
    self.add_new_transition(start, input, accept);
    NFABasic {
      start,
      accept,
    }
  }

  fn construct_e(&mut self) -> NFABasic {
    self.construct_singleton(None)
  }

  fn construct_single_char(&mut self, chr: char) -> NFABasic {
    self.construct_singleton(Some(chr))
  }

  fn union(&mut self, nfa_a: NFABasic, nfa_b: NFABasic) -> NFABasic {
    let start = self.gen_new_state_idx();
    let accept = self.gen_new_state_idx();
    self.add_new_transition(start, None, nfa_a.start);
    self.add_new_transition(start, None, nfa_b.start);
    self.add_new_transition(nfa_a.accept, None, accept);
    self.add_new_transition(nfa_b.accept, None, accept);
    NFABasic {
      start,
      accept,
    }
  }

  fn concat(&mut self, nfa_a: NFABasic, nfa_b: NFABasic) -> NFABasic {
    self.add_new_transition(nfa_a.accept, None, nfa_b.start);
    NFABasic {
      start: nfa_a.start,
      accept: nfa_b.accept,
    }
  }

  fn closure_basic(
    &mut self,
    nfa: NFABasic,
    has_start_to_end: bool,
    has_inner_end_to_start: bool,
  ) -> NFABasic {
    let start = self.gen_new_state_idx();
    let end = self.gen_new_state_idx();
    self.add_new_transition(start, None, nfa.start);
    self.add_new_transition(nfa.accept, None, end);

    if has_start_to_end {
      self.add_new_transition(start, None, end);
    }
    if has_inner_end_to_start {
      self.add_new_transition(nfa.accept, None, nfa.start);
    }
    NFABasic {
      start,
      accept: end,
    }
  }

  fn closure(&mut self, nfa: NFABasic) -> NFABasic {
    self.closure_basic(nfa, true, true)
  }

  fn closure_plus(&mut self, nfa: NFABasic) -> NFABasic {
    self.closure_basic(nfa, false, true)
  }

  fn question_mark(&mut self, nfa: NFABasic) -> NFABasic {
    self.closure_basic(nfa, true, false)
  }
}

struct StackFrame {
  op_stack: Vec<RegOp>,
  item_stack: Vec<NFABasic>,
}

impl NFAOne {
  pub fn from_regexp(reg_exp: &str) -> Self {
    let mut nfa_constructor = NFAConstructor::new();
    let mut stack: Vec<StackFrame> = vec![StackFrame {
      op_stack: vec![RegOp::Eof],
      item_stack: vec![]
    }];

    fn reduce_frame(frame: &mut StackFrame, nfa_constructor: &mut NFAConstructor, new_op: RegOp) {
      while let Some(top_op) = frame.op_stack.pop() {
        if top_op.get_priority() < new_op.get_priority() {
          frame.op_stack.push(top_op); // push back, top_op has lower priority
          break;
        }
        match top_op {
          RegOp::Eof | RegOp::Paren => {}, // do nothing
          RegOp::Union | RegOp::Concat => { // binary operator
            let operand_right = frame.item_stack.pop().expect(&format!("parse fail at {:?}", top_op));
            let operand_left = frame.item_stack.pop().expect(&format!("parse fail at {:?}", top_op));
            frame.item_stack.push(
              match top_op {
                RegOp::Union => nfa_constructor.union(operand_left, operand_right),
                RegOp::Concat => nfa_constructor.concat(operand_left, operand_right),
                _ => unreachable!(),
              }
            );
          },
          RegOp::Closure | RegOp::Plus | RegOp::Question => {
            let operand = frame.item_stack.pop().expect(&format!("parse fail at {:?}", top_op));
            frame.item_stack.push(
              match top_op {
                RegOp::Closure => nfa_constructor.closure(operand),
                RegOp::Plus => nfa_constructor.closure_plus(operand),
                RegOp::Question => nfa_constructor.question_mark(operand),
                _ => unreachable!(),
              }
            );
          },
        }
      }
      frame.op_stack.push(new_op);
    }

    let mut is_last_reg_item = false;
    for chr in EscapeChars::new(reg_exp.chars()) {
      match chr {
        MaybeEsc::NonEsc('(') => {
          if is_last_reg_item { 
            reduce_frame(stack.last_mut().unwrap(), &mut nfa_constructor, RegOp::Concat);
          }
          stack.push(StackFrame { op_stack: vec![RegOp::Paren], item_stack: vec![] });
          is_last_reg_item = false;
        },
        MaybeEsc::NonEsc(')') => {
          let mut current_frame = stack.pop().expect("parse error at )");
          reduce_frame(&mut current_frame, &mut nfa_constructor, RegOp::Paren);
          assert!(current_frame.item_stack.len() <= 1, "current frame error");
          let frame_res = if current_frame.item_stack.is_empty() {
            nfa_constructor.construct_e()
          } else {
            current_frame.item_stack.pop().unwrap()
          };
          stack.last_mut().expect("stack empty error").item_stack.push(frame_res);
          is_last_reg_item = true;
        },
        MaybeEsc::NonEsc('|') => {
          reduce_frame(stack.last_mut().unwrap(), &mut nfa_constructor, RegOp::Union);
          is_last_reg_item = false;
        },
         MaybeEsc::NonEsc('*')
         | MaybeEsc::NonEsc('?')
         | MaybeEsc::NonEsc('+') => {
          reduce_frame(stack.last_mut().unwrap(), &mut nfa_constructor, match chr.get_chr() {
            '*' => RegOp::Closure,
            '?' => RegOp::Question,
            '+' => RegOp::Plus,
            _ => unreachable!(),
          });
          is_last_reg_item = true;
        },
        maybe_esc_chr => { // alphabet like a,b,c,d
          let chr = maybe_esc_chr.get_chr();
          if is_last_reg_item { 
            reduce_frame(stack.last_mut().unwrap(), &mut nfa_constructor, RegOp::Concat);
          }
          stack.last_mut().expect("no stack frame").item_stack.push(nfa_constructor.construct_single_char(chr));
          is_last_reg_item = true;
        }
      }
    }
    // finally reduce frame with RegOp::Eof

    assert!(stack.len() == 1, "parse error: stack should be reduced to 1");
    let mut stack_frame = stack.pop().unwrap();
    reduce_frame(&mut stack_frame, &mut nfa_constructor, RegOp::Eof);

    assert!(stack_frame.item_stack.len() <= 1);
    let res = if stack_frame.item_stack.is_empty() {
      nfa_constructor.construct_e()
    } else {
      stack_frame.item_stack.pop().unwrap()
    };
    NFAOne {
      states_size: nfa_constructor.state_idx,
      start: res.start,
      accept: vec![res.accept],
      transition_func: Box::new(move |state: usize, input: Option<char>| {
        match nfa_constructor.transition_map.get(&(state, input)) {
          Some(states) => states.clone(),
          None => vec![],
        }
      })
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn regexp_instance_1() {
    let regexp = NFAOne::from_regexp("(a|b)*abb");
    assert!(regexp.test("ababb"));
    assert!(!regexp.test("abab"));
    assert!(regexp.test("abababababababb"));
    assert!(regexp.test("abb"));
    assert!(!regexp.test("ab"));
  }

  #[test]
  fn regexp_instance_2() {
    let regexp = NFAOne::from_regexp("(a|bc)*abb");
    assert!(regexp.test("abcabb"));
    assert!(regexp.test("aabb"));
    assert!(regexp.test("bcabb"));
    assert!(regexp.test("abcaabb"));
    assert!(regexp.test("abcbcaabb"));
    assert!(regexp.test("abcbcabcaabb"));
    assert!(!regexp.test("abcbcabcaabbc"));
    assert!(!regexp.test("abcbcabbc"));
  }

  #[test]
  fn regexp_number() {
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
  }
}