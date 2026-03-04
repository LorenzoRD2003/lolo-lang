/*
  Responsabilidad única:
  - Conoce el source, donde estan los newlines, y traduce offsets

  Funciones tipicas:
  offset -> (line, column)
  span -> (line_start, col_start, line_end, col_end)

  NOTA: Por convencion, decimos que un '\n' ocupa la columna 0 de la nueva linea.
*/

use crate::common::span::Span;

#[derive(Debug, Clone)]
pub struct SourceMap<'a> {
  source: &'a str,
  file_name: &'a str,
  newlines: Vec<usize>,
}

impl<'a> SourceMap<'a> {
  pub fn new(source: &'a str, file_name: &'a str) -> Self {
    let newlines = source
      .bytes()
      .enumerate()
      .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
      .collect();
    Self {
      source,
      file_name,
      newlines,
    }
  }

  pub(crate) fn source(&self) -> &'a str {
    self.source
  }

  pub(crate) fn file_name(&self) -> &'a str {
    self.file_name
  }

  pub(crate) fn offset_to_line_column(&self, offset: usize) -> (usize, usize) {
    let line = match self.newlines.binary_search(&offset) {
      Ok(idx) => idx + 1,
      Err(idx) => idx + 1,
    };
    let line_start = if line == 1 {
      0
    } else {
      self.newlines[line - 2] + 1
    };
    let column = offset - line_start + 1;
    (line, column)
  }

  pub(crate) fn span_to_line_column(&self, span: &Span) -> (usize, usize, usize, usize) {
    let (line_start, column_start) = self.offset_to_line_column(span.start);
    let (line_end, column_end) = self.offset_to_line_column(span.end);
    (line_start, column_start, line_end, column_end)
  }

  pub(crate) fn get_nth_line(&self, line: usize) -> Option<Span> {
    if self.source.len() == 0 || line == 0 || line > self.newlines.len() + 1 {
      return None;
    }
    let span_start = if line == 1 {
      0
    } else {
      self.newlines[line - 2] + 1
    };
    let span_end = if line == self.newlines.len() + 1 {
      self.source.len()
    } else {
      self.newlines[line - 1]
    };
    Some(span_start..span_end)
  }
}

#[cfg(test)]
mod tests;
