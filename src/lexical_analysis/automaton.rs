pub trait Automaton {
  type State;

  fn init_state(&self) -> Self::State;
  fn is_dead(&self, s: &Self::State) -> bool;
  fn is_accept(&self, s: &Self::State) -> bool;
  fn transition(&self, s: &Self::State, chr: char) -> Self::State;

  fn test(&self, s: &str) -> bool {
    let mut state = self.init_state();
    if self.is_dead(&state) {
      return false;
    }
    for chr in s.chars() {
      state = self.transition(&state, chr);
      if self.is_dead(&state) {
        return false;
      }
    }
    self.is_accept(&state)
  }
}

struct AutomatonRunner<T: Automaton> {
  curr_state: T::State,
  automaton: T,
}
   
impl <T: Automaton> AutomatonRunner<T> {
  fn new(automaton: T) -> Self {
    AutomatonRunner {
      curr_state: automaton.init_state(),
      automaton,
    }
  }

  fn is_dead(&self) -> bool {
    self.automaton.is_dead(&self.curr_state)
  }
 
  fn is_accept(&self) -> bool {
    self.automaton.is_accept(&self.curr_state)
  }

  fn next(&mut self, chr: char) {
    self.curr_state = self.automaton.transition(&self.curr_state, chr);
  }
}