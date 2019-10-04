use super::automaton::{Automaton};
use std::collections::{HashMap, HashSet};

pub struct DFAOne {
  pub states_size: usize,
  pub start: usize,
  pub accept: Vec<usize>,
  pub transition_func: Box<dyn Fn(usize, char) -> usize>,
}

struct Partition {
  id_group_map: Vec<usize>,
  group_ids_map: Vec<Vec<usize>>,
}

impl Partition {
  fn new(size: usize) -> Self {
    Partition {
      id_group_map: vec![0; size],
      group_ids_map: vec![(0..size).collect()],
    }
  }
  
  fn split(&mut self, group: usize, new_color: &[usize]) {
    assert_eq!(self.group_ids_map[group].len(), new_color.len(), "new_color map must matches group len");
    let mut color_ids_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for (&id, &color) in self.group_ids_map[group].iter().zip(new_color.iter()) {
      color_ids_map.entry(color).or_insert(vec![]).push(id);
    }
    let mut color_ids_map_iter = color_ids_map.into_iter();
    let (_color, first_group_ids) = color_ids_map_iter.next().expect("at least one group");
    self.group_ids_map[group] = first_group_ids;

    for (_color, other_groups_ids) in color_ids_map_iter {
      let group_idx = self.group_ids_map.len();
      for &id in &other_groups_ids {
        self.id_group_map[id] = group_idx;
      }
      self.group_ids_map.push(other_groups_ids);
    }
  }

  fn which_group(&self, id: usize) -> usize {
    self.id_group_map[id]
  }
}

fn has_at_least_two_elements(color: &[usize]) -> bool {
  for i in 1..color.len() {
    if color[i] != color[i-1] {
      return true;
    }
  }
  false
}

impl DFAOne {
  fn state_minimization(&self, input: &str) -> DFAOne {
    let mut parti = Partition::new(self.states_size);
    // first partition accorinding to accept
    let mut new_color = vec![0; self.states_size];
    for &accept_s in &self.accept {
      new_color[accept_s] = 1;
    }
    parti.split(0, &new_color);

    loop {
      let mut has_new_parti = false;
      for group in 0..parti.group_ids_map.len() {
        for chr in input.chars() {
          let mut color = vec![];
          for &id in &parti.group_ids_map[group] {
            color.push((self.transition_func)(id, chr));
          }
          if has_at_least_two_elements(&color) {
            has_new_parti = true;
            parti.split(group, &color);
          }
        }
      }
      if !has_new_parti {
        break;
      }
    }

    let new_states_size = parti.group_ids_map.len();
    let new_accept: Vec<_> = self.accept
      .iter()
      .map(|&s| parti.which_group(s))
      .collect::<HashSet<_>>()
      .into_iter()
      .collect();
    let new_start = parti.which_group(self.start);
    let mut new_transition_map: HashMap<char, Vec<usize>> = HashMap::new();
    for chr in input.chars() {
      let ts: Vec<_> = (0..new_states_size)
        .map(|s| parti.which_group((self.transition_func)(parti.group_ids_map[s][0], chr)))
        .collect();
      new_transition_map.insert(chr, ts);
    }

    DFAOne {
      states_size: new_states_size,
      start: new_start,
      accept: new_accept,
      transition_func: Box::new(move |s: usize, chr: char| {
        new_transition_map.get(&chr).expect("invalid input")[s]
      })
    }
  }
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
      states_size: 4,
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
    assert!(dfa.test("abb"));
    assert!(dfa.test("abababaabb"));
  }


  #[test]
  fn partition_works() {
    let mut parti_1 = Partition::new(4);
    assert_eq!(parti_1.id_group_map, vec![0,0,0,0]);
    assert_eq!(parti_1.group_ids_map, vec![vec![0,1,2,3]]);
    parti_1.split(0, &[1,1,3,4]);
    assert_eq!(parti_1.which_group(0), parti_1.which_group(1));
    assert_ne!(parti_1.which_group(0), parti_1.which_group(2));
    assert_ne!(parti_1.which_group(0), parti_1.which_group(3));
    assert_ne!(parti_1.which_group(2), parti_1.which_group(3));
  }

  #[test]
  fn state_minimization_works() {
    let dfa = DFAOne {
      states_size: 5,
      start: 0,
      accept: vec![4],
      transition_func: Box::new(|state: usize, chr: char| {
        match chr {
          'a' => [1,1,1,1,1][state],
          'b' => [2,3,2,4,2][state],
          _ => panic!("no {} in current language", chr),
        }
      }),
    };
    let min_dfa = dfa.state_minimization("ab");
    assert_eq!(min_dfa.states_size, 4);
    assert!(min_dfa.test("aabb"));
    assert!(!min_dfa.test("abbb"));
    assert!(min_dfa.test("abb"));
    assert!(min_dfa.test("abababaabb"));

    let minmin_dfa = min_dfa.state_minimization("ab");
    assert_eq!(minmin_dfa.states_size, 4);
    assert!(minmin_dfa.test("aabb"));
    assert!(!minmin_dfa.test("abbb"));
    assert!(minmin_dfa.test("abb"));
    assert!(minmin_dfa.test("abababaabb"));
  }
}

