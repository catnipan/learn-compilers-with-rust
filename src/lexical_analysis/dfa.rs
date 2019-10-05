use super::automaton::Automaton;
use std::collections::{HashMap, HashSet};

pub struct DFAOne {
  pub states_size: usize,
  pub start: Option<usize>,
  pub accept: Vec<usize>,
  pub transition_func: Box<dyn Fn(usize, char) -> Option<usize>>,
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
    let mut parti = Partition::new(self.states_size + 1); // one more state for the dead one
    // first partition accorinding to accept
    let mut new_color = vec![0; self.states_size + 1];
    for &accept_s in &self.accept {
      new_color[accept_s] = 1;
    }
    let dead_state_id = self.states_size;
    parti.split(0, &new_color);

    let enhanced_transition_func = |s: usize, chr: char| {
      if s == dead_state_id {
        dead_state_id
      } else {
        (self.transition_func)(s, chr).unwrap_or(dead_state_id)
      }
    };

    loop {
      let mut has_new_parti = false;
      for group in 0..parti.group_ids_map.len() {
        if parti.group_ids_map[group].len() == 1 { // optimize when group can't be split
          continue;
        }
        for chr in input.chars() {
          let mut color = vec![];
          for &id in &parti.group_ids_map[group] {
            let to_id = enhanced_transition_func(id, chr);
            color.push(parti.which_group(to_id));
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

    let mut new_transition_map: HashMap<char, Vec<Option<usize>>> = HashMap::new();
    
    // generate old_state => new_state map (omitting dead state)
    let mut new_state_old_represent = vec![];
    for (group, old_ids) in parti.group_ids_map.iter().enumerate() {
      if old_ids[0] == dead_state_id {
        continue;
      }
      new_state_old_represent.push(old_ids[0]);
    }
    let dead_group = parti.which_group(dead_state_id);
    let enhanced_which_group = |old_s: usize| {
      let w = parti.which_group(old_s);
      if w > dead_group {
        w - 1
      } else {
        w
      }
    };
    let new_accept: Vec<_> = self.accept
      .iter()
      .map(|&s| enhanced_which_group(s))
      .collect::<HashSet<_>>()
      .into_iter()
      .collect();
    let new_start = self.start.map(|i| enhanced_which_group(i));

    for chr in input.chars() {
      let ts: Vec<_> = new_state_old_represent
        .iter()
        .map(|&old_rs| {
          (self.transition_func)(old_rs, chr).map(|old_ts| enhanced_which_group(old_ts))
        })
        .collect();
      new_transition_map.insert(chr, ts);
    }


    DFAOne {
      states_size: new_state_old_represent.len(),
      start: new_start,
      accept: new_accept,
      transition_func: Box::new(move |s: usize, chr: char| {
        new_transition_map.get(&chr).and_then(|to_s_map| to_s_map[s])
      })
    }
  }
}

impl Automaton for DFAOne {
  type State = Option<usize>;
  fn init_state(&self) -> Self::State {
    self.start
  }
  fn is_dead(&self, s: &Self::State) -> bool {
    s.is_none()
  }
  fn is_accept(&self, s: &Self::State) -> bool {
    match s {
      Some(s) => self.accept.contains(s),
      None => false,
    }
  }
  fn transition(&self, s: &Self::State, chr: char) -> Self::State {
    s.and_then(|s| (self.transition_func)(s, chr))
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
      start: Some(0),
      accept: vec![3],
      transition_func: Box::new(|state: usize, chr: char| {
        match chr {
          'a' => Some([1,1,1,1][state]),
          'b' => Some([0,2,3,0][state]),
          _ => None,
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
      start: Some(0),
      accept: vec![4],
      transition_func: Box::new(|state: usize, chr: char| {
        match chr {
          'a' => Some([1,1,1,1,1][state]),
          'b' => Some([2,3,2,4,2][state]),
          _ => None,
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

