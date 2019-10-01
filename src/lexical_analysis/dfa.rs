trait Automaton {
  fn test(&self, s: &str) -> bool;
}

struct DFAOne {
  start: usize,
  accept: Vec<usize>,
  transition_func: Box<dyn Fn(usize, char) -> usize>,
}

impl Automaton for DFAOne {
  fn test(&self, s: &str) -> bool {
    let mut curr_state = self.start;
    for chr in s.chars() {
      curr_state = (self.transition_func)(curr_state, chr);
    }
    self.accept.contains(&curr_state)
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_instance_1() {
    // (a|b)*abb
    let dfa = DFAOne {
      start: 0,
      accept: vec![3],
      transition_func: Box::new(|state: usize, chr: char| {
        match chr {
          'a' => [1,1,1,1][state],
          'b' => [0,2,3,0][state],
          _ => panic!("no {} in current language", chr),
        }
      }),
    };
    assert!(dfa.test("aabb"));
    assert!(!dfa.test("abbb"));
    assert!(dfa.test("abababaabb"));
  }
}

