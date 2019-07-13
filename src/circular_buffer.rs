use std::collections::vec_deque::VecDeque;

pub struct CircularBuffer<T> {
  max_size: usize,
  dqueue: VecDeque<T>,
}

impl <T> CircularBuffer<T> {
  pub fn new(max_size: usize) -> CircularBuffer<T> {
    CircularBuffer {
      max_size: max_size,
      dqueue: VecDeque::with_capacity(max_size)
    }
  }

  pub fn add(&mut self, value: T) {
    if self.dqueue.len() == self.max_size {
      self.dqueue.pop_front();
    }
    self.dqueue.push_back(value);
  }

  pub fn iter(&self) -> std::collections::vec_deque::Iter<T> {
    self.dqueue.iter()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dont_overflow() {
    let mut buf = CircularBuffer::new(3);
    buf.add("unu");
    buf.add("doi");
    let mut i = buf.iter();
    assert_eq!(Some(&"unu"), i.next());
    assert_eq!(Some(&"doi"), i.next());
    assert_eq!(None, i.next());
    // assert_eq!(2, count_items(buf));
  }

  #[test]
  fn keep_last_added() {
    let mut buf = CircularBuffer::new(2);
    buf.add("unu");
    buf.add("doi");
    buf.add("trei");
    buf.add("patru");
    
    let mut i = buf.iter();
    assert_eq!(Some(&"trei"), i.next());
    assert_eq!(Some(&"patru"), i.next());
    assert_eq!(None, i.next());
  }
}