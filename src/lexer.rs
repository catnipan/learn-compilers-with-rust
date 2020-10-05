use super::automaton::Automaton;
use std::iter::Iterator;

struct Lexer<A: Automaton, Lexeme>(Vec<(A, Box<dyn Fn(&str) -> Lexeme>)>);

impl<A: Automaton, Lexeme> Lexer<A, Lexeme> {
  fn parse(&self, input: String) -> LexerRunner<A, Lexeme> {
    LexerRunner {
      input,
      input_idx: 0,
      lexer: &self,
    }
  }
}

struct LexerRunner<'a, A: Automaton, Lexeme> {
  input: String,
  input_idx: usize,
  lexer: &'a Lexer<A, Lexeme>,
}

impl<'a, A: Automaton, Lexeme> Iterator for LexerRunner<'a, A, Lexeme> {
  type Item = Lexeme;
  fn next(&mut self) -> Option<Self::Item> {
    if self.input_idx == self.input.len() {
      return None;
    }
    let mut state = vec![];
    let mut last_priority_action_idx: Option<(usize, usize)> = None;
    for (lidx, (l, _action)) in self.lexer.0.iter().enumerate() {
      let s = l.init_state();
      if !l.is_dead(&s) {
        state.push((lidx, s));
      }
    }
    for curr_input_idx in self.input_idx..self.input.len() {
      let chr = self.input.as_bytes()[curr_input_idx] as char;
      let mut next_state = vec![];
      let mut curr_priority_lidx = None;
      for (lidx, s) in state {
        let ref lexer = self.lexer.0[lidx].0;
        let new_s = lexer.transition(&s, chr);
        if lexer.is_dead(&new_s) {
          continue;
        }
        if lexer.is_accept(&new_s) {
          if curr_priority_lidx.is_none() {
            curr_priority_lidx = Some(lidx);
          }
        }
        next_state.push((lidx, new_s));
      }
      if let Some(lidx) = curr_priority_lidx {
        last_priority_action_idx = Some((curr_input_idx, lidx));
      }
      if next_state.is_empty() {
        break;
      }
      state = next_state;
    }

    match last_priority_action_idx {
      Some((input_end_idx, lidx)) => {
        let new_input_idx = input_end_idx + 1;
        let lexeme = &self.input[self.input_idx..new_input_idx];
        self.input_idx = new_input_idx;
        let token = (self.lexer.0[lidx].1)(lexeme);
        Some(token)
      },
      None => None, // parse ended
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use super::super::dfa::DFAOne;
  use super::super::dfa_regexp;

  #[test]
  fn arithmetic_lexeme() {
    #[derive(Debug, PartialEq, Eq)]
    enum Token {
      Number(u32),
      Plus,
      Subtract,
      Multiply,
      Divide,
      LeftParen,
      RightParen,
    }

    let lexer = Lexer(vec![
      (DFAOne::from_regexp("(1|2|3|4|5|6|7|8|9)(0|1|2|3|4|5|6|7|8|9)*|0", "0123456789"), Box::new(|num: &str| Token::Number(num.parse::<u32>().expect("number parse fail")))),
      (DFAOne::from_regexp("\\+", "\\+"), Box::new(|_| Token::Plus)),
      (DFAOne::from_regexp("-", "-"), Box::new(|_| Token::Subtract)),
      (DFAOne::from_regexp("\\*", "\\*"), Box::new(|_| Token::Multiply)),
      (DFAOne::from_regexp("/", "/"), Box::new(|_| Token::Divide)),
      (DFAOne::from_regexp("\\(", "\\("), Box::new(|_| Token::LeftParen)),
      (DFAOne::from_regexp("\\)", "\\)"), Box::new(|_| Token::RightParen)),
    ]);

    assert_eq!(
      lexer.parse("12+35".to_string()).collect::<Vec<_>>(),
      &[Token::Number(12), Token::Plus, Token::Number(35)]
    );
    assert_eq!(
      lexer.parse("1+23-(3*45/5)".to_string()).collect::<Vec<_>>(),
      &[Token::Number(1), Token::Plus, Token::Number(23), Token::Subtract, Token::LeftParen, Token::Number(3), Token::Multiply, Token::Number(45), Token::Divide, Token::Number(5), Token::RightParen]
    );
  }
}
