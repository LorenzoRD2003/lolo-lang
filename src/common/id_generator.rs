use std::marker::PhantomData;

pub(crate) trait IncrementalId: Sized {
  fn from_usize(value: usize) -> Self;
}

pub(crate) trait IdGenerator {
  type Id;
  fn next_id(&mut self) -> Self::Id;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct IncrementalIdGenerator<T> {
  current: usize,
  _marker: PhantomData<T>,
}

impl<T> IncrementalIdGenerator<T> {
  pub(crate) fn new() -> Self {
    Self {
      current: 0,
      _marker: PhantomData,
    }
  }
}

impl<T> IdGenerator for IncrementalIdGenerator<T>
where
  T: IncrementalId,
{
  type Id = T;

  fn next_id(&mut self) -> Self::Id {
    let id = T::from_usize(self.current);
    self.current += 1;
    id
  }
}
