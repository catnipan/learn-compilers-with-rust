#[derive(Copy, Clone, Debug)]
pub enum RegOp { Eof, Paren, Union, Concat, Closure, Plus, Question }

impl RegOp {
  pub fn get_priority(&self) -> i32 {
    match self {
      RegOp::Eof => 0,
      RegOp::Paren => 1,
      RegOp::Union => 2,
      RegOp::Concat => 3,
      RegOp::Closure | RegOp::Plus | RegOp::Question => 4,
    }
  }
}