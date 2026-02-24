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

#[derive(Debug, Clone, Copy)]
pub(crate) enum LabelStyle {
  Primary,
  Secondary,
}

#[derive(Debug, Clone)]
pub(crate) struct Label {
  span: Span,
  message: Option<String>,
  style: LabelStyle,
}
