use std::io::prelude::{Read};

#[derive(Debug)]
pub struct InputBuffer<Input: Read> {
  input: Input,
  size: usize,
  forward: (usize, usize),
  buffers: [Vec<u8>; 2],
}

impl<Input: Read> InputBuffer<Input> {
  pub fn new(input: Input, size: usize) -> Self {
    assert!(size >= 2, "buffer size must >= 2");
    let mut new_buffer = InputBuffer {
      input,
      size,
      forward: (0, 0),
      buffers: [vec![0; size], vec![0; size]],
    };
    new_buffer.load_buffer(0);
    new_buffer
  }

  fn load_buffer(&mut self, buffer_idx: usize) {
    let read_len = self.input.read(&mut self.buffers[buffer_idx][0..self.size - 1]).expect("read failed");
    self.buffers[buffer_idx][read_len] = 0;
  }

  fn next(&mut self) -> u8 {
    let (buffer_idx, idx) = self.forward;
    match self.buffers[buffer_idx][idx] {
      0 if idx == self.size - 1 => {
        self.load_buffer(1 - buffer_idx);
        self.forward = (1 - buffer_idx, 0);
        self.next()
      },
      0 => 0,
      o@_ => {
        self.forward.1 += 1;
        o
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn routine(input: &str, buffer_size: usize) {
    let mut buffer = InputBuffer::new(input.as_bytes(), buffer_size);
    let mut output = String::new();
    loop {
      let n = buffer.next();
      if n == 0 { break; }
      output.push(n as char);
    }
    assert_eq!(input, &output);
  }

  #[test]
  fn output_equals_input() {
    routine("hello rust hello world!", 4);
    routine("hello rust hello world!", 3);
    routine("hello rust hello world!", 2);
  }
}

