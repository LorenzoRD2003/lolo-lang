use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

trait IncrementalId: Sized {
  fn from_usize(value: usize) -> Self;
}

impl IncrementalId for ScopeId {
  fn from_usize(value: usize) -> Self {
    ScopeId(value)
  }
}

impl IncrementalId for SymbolId {
  fn from_usize(value: usize) -> Self {
    SymbolId(value)
  }
}

pub trait IdGenerator {
  type Id;
  fn next_id(&mut self) -> Self::Id;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IncrementalIdGenerator<T> {
  current: usize,
  _marker: PhantomData<T>,
}

impl<T> IncrementalIdGenerator<T> {
  pub fn new() -> Self {
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
