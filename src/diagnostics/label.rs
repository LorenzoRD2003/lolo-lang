// Un diagnostic puede tener span principal y spans auxiliares, porque un error puede involucrar varios lugares
// Ejemplo:
// error: use of undefined variable
//   --> main.lolo:5:10
// note: variable referenced here
// note: declaration missing
// un Label permite errores tipo Rustc, al modelar:
// - span
// - mensaje opcional
// - estilo (primary / secondary) -> esto tambien forma parte de la semantica del label

use crate::common::span::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelStyle {
  Primary,
  Secondary,
}

#[derive(Debug, Clone)]
pub struct Label {
  pub span: Span,
  pub message: Option<String>,
  pub style: LabelStyle,
}

impl Label {
  pub fn primary(span: Span, msg: Option<String>) -> Self {
    Self {
      span,
      message: msg,
      style: LabelStyle::Primary,
    }
  }

  pub fn secondary(span: Span, msg: Option<String>) -> Self {
    Self {
      span,
      message: msg,
      style: LabelStyle::Secondary,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn label_primary_secondary() {
    let primary = Label::primary(1..3, Some("error".into()));
    let secondary = Label::secondary(4..6, None);

    assert_eq!(primary.style, LabelStyle::Primary);
    assert_eq!(secondary.style, LabelStyle::Secondary);
    assert_eq!(primary.message.as_deref(), Some("error"));
    assert!(secondary.message.is_none());
  }
}
