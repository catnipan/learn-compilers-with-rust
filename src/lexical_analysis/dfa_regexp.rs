use super::regop::RegOp;
use super::dfa::DFAOne;
use super::automaton::Automaton;
use std::collections::{HashSet, HashMap};

#[derive(Copy, Clone, Debug)]
pub enum NodeType {
  Closure,
  Concat,
  Union,
}

#[derive(Clone, Debug)]
pub enum RegASTNode {
  Endmarker,
  LeafEmpty,
  Leaf(char),
  Unary { node_type: NodeType, child: Box<RegASTNode> },
  Binary { node_type: NodeType, left_child: Box<RegASTNode>, right_child: Box<RegASTNode> },
}

struct StackFrame {
  op_stack: Vec<RegOp>,
  item_stack: Vec<RegASTNode>,
}

pub fn parse_ast_regexp(reg_exp: &str) -> RegASTNode {
  let mut stack: Vec<StackFrame> = vec![];
  let mut curr_stackframe = StackFrame {
    op_stack: vec![],
    item_stack: vec![]
  };

  fn push_new_op(curr_stackframe: &mut StackFrame, new_op: RegOp) {
    while let Some(top_op) = curr_stackframe.op_stack.pop() {
      if top_op.get_priority() < new_op.get_priority() {
        curr_stackframe.op_stack.push(top_op); // push back, top_op has lower priority
        break;
      }
      match top_op {
        RegOp::Eof | RegOp::Paren => {}, // do nothing
        RegOp::Union | RegOp::Concat => { // binary operator
          let operand_right = curr_stackframe.item_stack.pop().expect(&format!("parse fail at {:?}", top_op));
          let operand_left = curr_stackframe.item_stack.pop().expect(&format!("parse fail at {:?}", top_op));
          curr_stackframe.item_stack.push(
            RegASTNode::Binary {
              node_type: match top_op {
               RegOp::Union => NodeType::Union,
                RegOp::Concat => NodeType::Concat,
                _ => unreachable!(),
              },
              left_child: Box::new(operand_left),
              right_child: Box::new(operand_right),
            }
          );
        },
        RegOp::Closure | RegOp::Plus | RegOp::Question => {
          let operand = curr_stackframe.item_stack.pop().expect(&format!("parse fail at {:?}", top_op));
          curr_stackframe.item_stack.push(
            match top_op {
              RegOp::Closure => RegASTNode::Unary {
                node_type: NodeType::Closure,
                child: Box::new(operand),
              },
              RegOp::Plus => RegASTNode::Binary {
                node_type: NodeType::Concat,
                left_child: Box::new(operand.clone()),
                right_child: Box::new(RegASTNode::Unary {
                  node_type: NodeType::Closure,
                  child: Box::new(operand),
                })
              },
              RegOp::Question => RegASTNode::Binary {
                node_type: NodeType::Union,
                left_child: Box::new(RegASTNode::LeafEmpty),
                right_child: Box::new(operand),
              },
              _ => unreachable!(),
            }
          );
        },
      }
    }
    curr_stackframe.op_stack.push(new_op);
  }

  let mut is_last_item = false;
  for chr in reg_exp.chars() {
    match chr {
      '(' => {
        if is_last_item { push_new_op(&mut curr_stackframe, RegOp::Concat); }
        stack.push(curr_stackframe);
        curr_stackframe = StackFrame {
          op_stack: vec![RegOp::Paren],
          item_stack: vec![]
        };
        is_last_item = false;
      },
      ')' => {
        push_new_op(&mut curr_stackframe, RegOp::Paren);
        assert!(curr_stackframe.item_stack.len() <= 1, "current frame error");
        let frame_res = if curr_stackframe.item_stack.is_empty() {
          RegASTNode::LeafEmpty
        } else {
          curr_stackframe.item_stack.pop().unwrap()
        };
        curr_stackframe = stack.pop().expect("stack empty error");
        curr_stackframe.item_stack.push(frame_res);
        is_last_item = true;
      },
      '|' => {
        push_new_op(&mut curr_stackframe, RegOp::Union);
        is_last_item = false;
      },
      '*' | '?' | '+' => {
        push_new_op(&mut curr_stackframe, match chr {
          '*' => RegOp::Closure,
          '?' => RegOp::Question,
          '+' => RegOp::Plus,
          _ => unreachable!(),
        });
        is_last_item = true;
      },
      chr => {
        if is_last_item { push_new_op(&mut curr_stackframe, RegOp::Concat); }
        curr_stackframe.item_stack.push(RegASTNode::Leaf(chr));
        is_last_item = true;
      },
    }
  }

  assert!(stack.is_empty(), "parse error: stack should be reduced to 1");
  push_new_op(&mut curr_stackframe, RegOp::Eof);
  assert!(curr_stackframe.item_stack.len() <= 1, "parse error: res itme should <= 1");
  RegASTNode::Binary {
    node_type: NodeType::Concat,
    left_child: Box::new(if curr_stackframe.item_stack.is_empty() {
      RegASTNode::LeafEmpty
    } else {
      curr_stackframe.item_stack.pop().unwrap()
    }),
    right_child: Box::new(RegASTNode::Endmarker),
  }
}

#[derive(Debug)]
struct DFABuilder {
  pos_idx: usize,
  follow_pos: Vec<HashSet<usize>>,
  pos_char_map: Vec<char>,
  end_idx: Option<usize>,
}

impl DFABuilder {
  fn new() -> Self {
    DFABuilder {
      pos_idx: 0,
      follow_pos: vec![],
      pos_char_map: vec![],
      end_idx: None,
    }
  }

  fn gen_new_idx(&mut self) -> usize {
    let res = self.pos_idx;
    self.pos_idx += 1;
    self.follow_pos.push(HashSet::new());
    res
  }

  fn acquire_new_pos_idx(&mut self, chr: char) -> usize {
    let new_idx = self.gen_new_idx();
    self.pos_char_map.push(chr);
    new_idx
  }

  fn acquire_end_pos_idx(&mut self) -> usize {
    let new_idx = self.gen_new_idx();
    assert!(self.end_idx.is_none(), "end index can only be set once");
    self.end_idx = Some(new_idx);
    self.pos_char_map.push('\0');
    new_idx
  }

  fn register_new_follow_pos(&mut self, ns: &[usize], fs: &[usize]) {
    for &n in ns {
      for &f in fs {
        self.follow_pos[n].insert(f);
      }
    }
  }
}

#[derive(Debug)]
struct TraverseInfo {
  nullable: bool,
  first_pos: Vec<usize>,
  last_pos: Vec<usize>,
}

impl TraverseInfo {
  fn new_empty() -> Self {
    TraverseInfo {
      nullable: true,
      first_pos: vec![],
      last_pos: vec![],
    }
  }

  fn new_singleton(idx: usize) -> Self {
    TraverseInfo {
      nullable: false,
      first_pos: vec![idx],
      last_pos: vec![idx],
    }
  }
}

struct RegExpDFA(DFAOne);

fn set_union(set_a: Vec<usize>, set_b: Vec<usize>) -> Vec<usize> {
  set_a.into_iter().chain(set_b.into_iter()).collect::<HashSet<usize>>().into_iter().collect()
}

impl RegExpDFA {
  fn new(reg_exp: &str, input: &str) -> Self {
    fn traverse_ast(node: &RegASTNode, builder: &mut DFABuilder) -> TraverseInfo {
      match node {
        RegASTNode::LeafEmpty => TraverseInfo::new_empty(),
        RegASTNode::Leaf(chr) => TraverseInfo::new_singleton(builder.acquire_new_pos_idx(*chr)),
        RegASTNode::Endmarker => TraverseInfo::new_singleton(builder.acquire_end_pos_idx()),
        RegASTNode::Unary { node_type, ref child } => {
          let child_info = traverse_ast(child, builder);
          match node_type {
            NodeType::Closure => {
              builder.register_new_follow_pos(&child_info.last_pos, &child_info.first_pos);
              TraverseInfo { nullable: true, ..child_info }
            },
            _ => unreachable!(),
          }
        },
        RegASTNode::Binary { node_type, ref left_child, ref right_child } => {
          let left_info = traverse_ast(left_child, builder);
          let right_info = traverse_ast(right_child, builder);
          match node_type {
            NodeType::Concat => {
              builder.register_new_follow_pos(&left_info.last_pos, &right_info.first_pos);
              TraverseInfo {
                nullable: left_info.nullable && right_info.nullable,
                first_pos: set_union(
                  left_info.first_pos,
                  if left_info.nullable { right_info.first_pos } else { vec![] }
                ),
                last_pos: set_union(
                  right_info.last_pos,
                  if right_info.nullable { left_info.last_pos } else { vec![] }
                ),
              }
            },
            NodeType::Union => TraverseInfo {
              nullable: left_info.nullable || right_info.nullable,
              first_pos: set_union(left_info.first_pos, right_info.first_pos),
              last_pos: set_union(left_info.last_pos, right_info.last_pos),
            },
            _ => unreachable!(),
          }
        },
      }
    }

    let mut dfa_builder = DFABuilder::new();
    let ast = parse_ast_regexp(reg_exp);
    println!("{:?}", ast);
    let root_info = traverse_ast(&ast, &mut dfa_builder);
    let end_idx = dfa_builder.end_idx.expect("invalid end marker");

    println!("{:?}", dfa_builder);

    let mut state_idx = 0;
    let mut states_idx_map: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut transition_map: HashMap<(usize, char), usize> = HashMap::new();
    let mut is_marked: Vec<bool> = vec![];
    let mut stack = vec![];

    let start_states = {
      let mut s = root_info.first_pos;
      s.sort_unstable();
      s
    };
    stack.push(start_states.clone());
    states_idx_map.insert(start_states, state_idx);
    is_marked.push(false);
    state_idx += 1;

    while let Some(curr_states) = stack.pop() {
      let &curr_idx = states_idx_map.get(&curr_states).expect(&format!("curr states {:?} unregistered", curr_states));
      if is_marked[curr_idx] {
        continue;
      }
      is_marked[curr_idx] = true;
      for chr in input.chars() {
        let mut res_states = vec![];
        for &pos in &curr_states {
          let pos_chr = dfa_builder.pos_char_map[pos];
          if pos_chr == chr {
            for &fpos in &dfa_builder.follow_pos[pos] {
              res_states.push(fpos);
            }
          }
        }
        let new_states = {
          res_states.sort_unstable();
          res_states.dedup();
          res_states
        };
        let new_state_idx = match states_idx_map.get(&new_states) {
          None => {
            let new_state_idx = state_idx;
            stack.push(new_states.clone());
            states_idx_map.insert(new_states, new_state_idx);
            is_marked.push(false);
            state_idx += 1;
            new_state_idx
          },
          Some(&idx) => idx,
        };
        transition_map.insert((curr_idx, chr), new_state_idx);
      }
    }

    let accept: Vec<usize> = states_idx_map.into_iter().filter_map(|(states, idx)| {
      match states.binary_search(&end_idx) {
        Ok(_) => Some(idx),
        Err(_) => None,
      }
    }).collect();

    RegExpDFA(DFAOne {
      start: 0,
      accept,
      transition_func: Box::new(move |s: usize, chr: char| {
        *transition_map.get(&(s, chr)).expect(&format!("invalid input {}", chr))
      })
    })
  }
}

impl Automaton for RegExpDFA {
  fn test(&self, test_str: &str) -> bool {
    self.0.test(test_str)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn regexp_instance_1() {
    let regexp = RegExpDFA::new("(a|b)*abb", "ab");
    assert!(regexp.test("ababb"));
    assert!(!regexp.test("abab"));
    assert!(regexp.test("abababababababb"));
    assert!(regexp.test("abb"));
    assert!(!regexp.test("ab"));
  }

  #[test]
  fn regexp_instance_2() {
    let regexp = RegExpDFA::new("(a|bc)*abb", "abc");
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
    let num_exp = RegExpDFA::new("(1|2|3|4|5|6|7|8|9)(0|1|2|3|4|5|6|7|8|9)*|0(.(0|1|2|3|4|5|6|7|8|9)+)?", "0123456789.");
    assert!(num_exp.test("0"));
    assert!(num_exp.test("4"));
    assert!(num_exp.test("10"));
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