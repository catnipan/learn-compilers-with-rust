
use std::io::prelude::{Read, Write};
use std::result::Result;

#[derive(Debug)]
pub enum ParseError {
  IoError,
  SyntaxError,
}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<Input: Read, Output: Write> {
  look_ahead: char,
  input: Input,
  output: Output,
}

impl<Input: Read, Output: Write> Parser<Input, Output> {
  pub fn new(input: Input, output: Output) -> Self {
    Parser {
      look_ahead: '\0',
      input,
      output,
    }
  }

  pub fn parse(&mut self) -> ParseResult<()> {
    self.look_ahead = self.read_one_char()?;
    self.expr()
  }

  fn expr(&mut self) -> ParseResult<()> {
    self.term()?;
    loop {
      match self.look_ahead {
        '+' => { self.match_lookahead('+')?; self.term()?; self.write_one_char('+')?; }
        '-' => { self.match_lookahead('-')?; self.term()?; self.write_one_char('-')?; }
        _ => break,
      }
    }
    Ok(())
  }

  fn term(&mut self) -> ParseResult<()> {
    if self.look_ahead.is_digit(10) {
      self.write_one_char(self.look_ahead)?;
      self.match_lookahead(self.look_ahead)
    } else {
      Err(ParseError::SyntaxError)
    }
  }

  fn read_one_char(&mut self) -> ParseResult<char> {
    let mut buffer = [0];
    match self.input.read(&mut buffer) {
      Ok(_) => Ok(buffer[0] as char),
      Err(_) => Err(ParseError::IoError),
    }
  }

  fn write_one_char(&mut self, t: char) -> ParseResult<()> {
    match self.output.write(&[t as u8]) {
      Ok(_) => Ok(()),
      Err(_) => Err(ParseError::IoError),
    }
  }

  fn match_lookahead(&mut self, t: char) -> ParseResult<()> {
    if self.look_ahead == t {
      self.look_ahead = self.read_one_char()?;
      Ok(())
    } else {
      Err(ParseError::IoError)
    }
  }
}

pub fn parse_io() {
  let mut parser = Parser::new(
    std::io::stdin(),
    std::io::stdout(),
  );
  parser.parse().expect("parse failed");
}

#[cfg(test)]
mod tests {
  use super::*;
  fn test_routine(source: &str, expected_result: &str) {
    let mut output: Vec<u8> = Vec::new();
    let mut parser = Parser::new(source.as_bytes(), &mut output);
    parser.parse().expect("parse failed");
    let result = String::from_utf8(output).expect("unvalid output");
    assert_eq!(result, expected_result);
  }

  #[test]
  fn it_produce_correct_result() {
    test_routine("3+4-5", "34+5-");
    test_routine("3-2", "32-");
    test_routine("3-2-4", "32-4-");
    test_routine("3-2-4-5", "32-4-5-");
    test_routine("3-2-4-5+6", "32-4-5-6+");
  }
}
