//! Responsabilidad: Definir Forward/Backward para el Dataflow Analysis

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
  Forward,
  Backward,
}

impl Direction {
  pub(crate) fn is_forward(self) -> bool {
    matches!(self, Self::Forward)
  }

  pub(crate) fn is_backward(self) -> bool {
    matches!(self, Self::Backward)
  }
}
