/*
  Responsabilidad única:
  - Conoce el source, donde estan los newlines, y traduce offsets

  Funciones tipicas:
  offset -> (line, column)
  span -> (line_start, col_start, line_end, col_end)

  NOTA: Por convencion, decimos que un '\n' ocupa la columna 0 de la nueva linea.
*/

use crate::common::span::Span;

pub struct SourceMap<'a> {
  source: &'a str,
  newlines: Vec<usize>,
}

impl<'a> SourceMap<'a> {
  fn new(source: &'a str) -> Self {
    let newlines = source
      .bytes()
      .enumerate()
      .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
      .collect();
    Self { source, newlines }
  }

  fn offset_to_line_column(&self, offset: usize) -> (usize, usize) {
    let line = match self.newlines.binary_search(&offset) {
      Ok(idx) => idx + 1,
      Err(idx) => idx + 1,
    };
    let line_start = if line == 1 {
      0
    } else {
      self.newlines[line - 2] + 1
    };
    dbg!(offset, line_start);
    let column = offset - line_start + 1;
    (line, column)
  }

  fn span_to_line_column(&self, span: Span) -> (usize, usize, usize, usize) {
    let (line_start, column_start) = self.offset_to_line_column(span.start);
    let (line_end, column_end) = self.offset_to_line_column(span.end);
    (line_start, column_start, line_end, column_end)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use proptest::{prop_assert, prop_assert_eq, proptest};

  #[test]
  fn empty_source_test() {
    let source_map = SourceMap::new("");
    assert_eq!((1, 1), source_map.offset_to_line_column(0));
  }

  #[test]
  fn single_character_source_test() {
    let source_map = SourceMap::new("a");
    assert_eq!((1, 1), source_map.offset_to_line_column(0));
    assert_eq!((1, 2), source_map.offset_to_line_column(1));
  }

  #[test]
  fn multiple_characters_without_newline_test() {
    let source_map = SourceMap::new("abcdef");
    assert_eq!((1, 1), source_map.offset_to_line_column(0));
    assert_eq!((1, 4), source_map.offset_to_line_column(3));
    assert_eq!((1, 7), source_map.offset_to_line_column(6));
  }

  #[test]
  fn offset_before_newline_test() {
    let source_map = SourceMap::new("abc\ndef");
    assert_eq!((1, 3), source_map.offset_to_line_column(2));
  }

  #[test]
  fn offset_in_newline_test() {
    let source_map = SourceMap::new("abc\ndef");
    assert_eq!((1, 4), source_map.offset_to_line_column(3));
  }

  #[test]
  fn offset_after_newline_test() {
    let source_map = SourceMap::new("abc\ndef");
    assert_eq!((2, 1), source_map.offset_to_line_column(4));
    assert_eq!((2, 2), source_map.offset_to_line_column(5));
  }

  #[test]
  fn multiple_newlines_test() {
    let source_map = SourceMap::new("a\nb\nc");
    assert_eq!((1, 1), source_map.offset_to_line_column(0));
    assert_eq!((1, 2), source_map.offset_to_line_column(1));
    assert_eq!((2, 1), source_map.offset_to_line_column(2));
    assert_eq!((2, 2), source_map.offset_to_line_column(3));
    assert_eq!((3, 1), source_map.offset_to_line_column(4));
    assert_eq!((3, 2), source_map.offset_to_line_column(5));
  }

  #[test]
  fn newline_at_end_test() {
    let source_map = SourceMap::new("abc\n");
    assert_eq!((1, 3), source_map.offset_to_line_column(2));
    assert_eq!((1, 4), source_map.offset_to_line_column(3));
    assert_eq!((2, 1), source_map.offset_to_line_column(4));
  }

  #[test]
  fn two_consecutive_newlines_test() {
    let source_map = SourceMap::new("abc\n\ndef");
    assert_eq!((1, 4), source_map.offset_to_line_column(3));
    assert_eq!((2, 1), source_map.offset_to_line_column(4));
    assert_eq!((3, 1), source_map.offset_to_line_column(5));
  }

  #[test]
  fn span_in_single_line_test() {
    let source_map = SourceMap::new("abc\ndef");
    assert_eq!((1, 2, 1, 3), source_map.span_to_line_column(1..2));
  }

  #[test]
  fn span_in_multiple_lines_test() {
    let source_map = SourceMap::new("abc\ndef");
    assert_eq!((1, 3, 2, 1), source_map.span_to_line_column(2..4));
  }

  #[test]
  fn complete_span_test() {
    let source = "abc\ndef";
    let source_map = SourceMap::new(source);
    assert_eq!(
      (1, 1, 2, 4),
      source_map.span_to_line_column(0..source.len())
    );
  }

  // monotonia de lineas
  proptest! {
    #[test]
    fn line_is_monotonic_test(source in ".*", a in 0usize..1000, b in 0usize..1000) {
      let sm = SourceMap::new(&source);
      let len = source.len();
      let a = a.min(len);
      let b = b.min(len);
      if a <= b {
        let (line_a, _) = sm.offset_to_line_column(a);
        let (line_b, _) = sm.offset_to_line_column(b);
        prop_assert!(line_a <= line_b);
      }
    }
  }

  // filas/columnas siempre >= 1
  proptest! {
    #[test]
    fn column_is_never_zero(source in ".*", offset in 0usize..1000) {
      let sm = SourceMap::new(&source);
      let offset = offset.min(source.len());
      let (line, col) = sm.offset_to_line_column(offset);
      prop_assert!(line >= 1 && col >= 1);
    }
  }

  // si no se cruzan lineas: offset + 1 -> columna + 1
  proptest! {
    #[test]
    fn column_progresses_correctly(source in ".*", offset in 0usize..1000) {
      let sm = SourceMap::new(&source);
      let len = source.len();
      if len == 0 { return Ok(()); }
      let offset = offset.min(len - 1);
      if source.as_bytes()[offset] != b'\n' {
        let (l1, c1) = sm.offset_to_line_column(offset);
        let (l2, c2) = sm.offset_to_line_column(offset + 1);
        if l1 == l2 {
          prop_assert_eq!(c2, c1 + 1);
        }
      }
    }
  }
}
