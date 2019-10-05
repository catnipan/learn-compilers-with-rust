use std::str::Chars;

pub struct EscapeChars<'a>(Chars<'a>);

impl<'a> EscapeChars<'a> {
  pub fn new(char_iterator: Chars<'a>) -> Self {
    EscapeChars(char_iterator)
  }
}

#[derive(PartialEq, Eq, Debug)]
pub enum MaybeEsc {
  Esc(char),
  NonEsc(char),
}

impl MaybeEsc {
  pub fn get_chr(&self) -> char {
    match self {
      MaybeEsc::Esc(chr) => *chr,
      MaybeEsc::NonEsc(chr) => *chr,
    }
  }
}

impl<'a> Iterator for EscapeChars<'a> {
  type Item = MaybeEsc;

  fn next(&mut self) -> Option<Self::Item> {
    match self.0.next() {
      Some('\\') => Some(MaybeEsc::Esc(self.0.next().expect("invalid string: ended in escaped character"))),
      Some(chr) => Some(MaybeEsc::NonEsc(chr)),
      None => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn maybeesc_works() {
    let mut esc = EscapeChars("h\\el\\lo".chars());
    assert_eq!(esc.next(), Some(MaybeEsc::NonEsc('h')));
    assert_eq!(esc.next(), Some(MaybeEsc::Esc('e')));
    assert_eq!(esc.next(), Some(MaybeEsc::NonEsc('l')));
    assert_eq!(esc.next(), Some(MaybeEsc::Esc('l')));
    assert_eq!(esc.next(), Some(MaybeEsc::NonEsc('o')));
    assert_eq!(esc.next(), None);
  }
}