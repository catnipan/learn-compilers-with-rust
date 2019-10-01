pub trait Automaton {
  fn test(&self, s: &str) -> bool;
}
