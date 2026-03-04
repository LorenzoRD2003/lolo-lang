// El renderer es el responsable de consumir diagnostics, pedir info al SourceMap, y es donde vive la logica de visualizacion
// - imprimir lineas relevantes
// - subrayar spans
// - alinear columnas

// Orden de renderizado estandar
// Severity + Message
// Location
// |
// | Code snippet
// |
// Labels
// Notes
//
// Ejemplo
// error: type mismatch
//  --> main.lolo:3:15
//   |
// 3 | let x = add 1 true;
//   |               ^^^ expected i32
//   |
// note: function defined here
// help: try converting types

use crate::{
  common::{SourceMap, Span},
  diagnostics::{diagnostic::Diagnostic, label::LabelStyle},
};
use std::fmt;

/// Renderizador usando spans.
/// Configuracion visual (colores, ascii/unicode, compact/verbose) -> por ahora no.
#[derive(Debug)]
pub struct Renderer<'a, W: fmt::Write> {
  /// SourceMap, para poder traducir los spans.
  source_map: &'a SourceMap<'a>,
  /// Output target, para indicar a donde van a renderizarse los mensajes.
  writer: W,
}

impl<'a, W: fmt::Write> Renderer<'a, W> {
  pub fn new(source_map: &'a SourceMap, writer: W) -> Self {
    Self { source_map, writer }
  }

  // Funcion responsable de todo el output
  pub fn render(&mut self, diag: &Diagnostic) -> fmt::Result {
    self.render_header(diag)?;
    self.render_location(diag)?;
    self.render_code_snippet(diag)?;
    self.render_notes(diag)
  }

  // Funcion responsable de renderizar la parte superior del error
  // error: undefined variable
  fn render_header(&mut self, diag: &Diagnostic) -> fmt::Result {
    writeln!(self.writer, "{}: {}", diag.severity(), diag.msg())
  }

  // Funcion responsable de renderizar la localizacion del error
  //  --> main.lolo:3:15
  fn render_location(&mut self, diag: &Diagnostic) -> fmt::Result {
    match diag.primary_span() {
      Some(span) => {
        let (line_start, column_start) = self.source_map.offset_to_line_column(span.start);
        writeln!(
          self.writer,
          " --> {}:{}:{}",
          self.source_map.file_name(),
          line_start,
          column_start
        )
      }
      None => writeln!(
        self.writer,
        " --> {}:unknown location",
        self.source_map.file_name()
      ),
    }
  }

  // Funcion responsable de:
  // - imprimir línea
  // - alinear
  // - subrayar
  // - multiline spans
  // - labels
  fn render_code_snippet(&mut self, diag: &Diagnostic) -> fmt::Result {
    match diag.primary_span() {
      Some(span) => {
        let source = self.source_map.source();
        let (line_start, _, line_end, _) = self.source_map.span_to_line_column(span);

        writeln!(self.writer, "  |")?;
        for cur_line in line_start..=line_end {
          // para cada linea que salga en el primary_span, vamos a mostrarla y subrayar la parte relevante debajo
          if let Some(Span {
            start: cur_line_start,
            end: cur_line_end,
          }) = self.source_map.get_nth_line(cur_line)
          {
            writeln!(
              self.writer,
              "{} | {}",
              cur_line,
              &source[cur_line_start..cur_line_end] // esto no se rompe porque mi lenguaje va a ser ASCII-only
            )?;
            let intersection_start = span.start.max(cur_line_start);
            let intersection_end = span.end.min(cur_line_end);
            let width = (intersection_end - intersection_start).max(1);
            let underline = " ".repeat(intersection_start - cur_line_start) + &"^".repeat(width);
            writeln!(self.writer, "  | {}", underline)?;
          }
        }

        Ok(())
      }
      None => Ok(()),
    }
  }

  /// Renderiza los labels secundarios (y primarios opcionales) de un Diagnostic
  fn render_labels(&mut self, diag: &Diagnostic) -> fmt::Result {
    for label in diag.labels() {
      let (line_start, _, line_end, _) = self.source_map.span_to_line_column(&label.span);

      // imprimimos cada línea del span
      for cur_line in line_start..=line_end {
        if let Some(Span {
          start: line_start_offset,
          end: line_end_offset,
        }) = self.source_map.get_nth_line(cur_line)
        {
          let intersection_start = label.span.start.max(line_start_offset);
          let intersection_end = label.span.end.min(line_end_offset);
          let width = (intersection_end - intersection_start).max(1);

          let underline_char = match label.style {
            LabelStyle::Primary => '^',
            LabelStyle::Secondary => '~',
          };
          let underline = " ".repeat(intersection_start - line_start_offset)
            + &underline_char.to_string().repeat(width);

          // escribimos la linea de subrayado
          writeln!(self.writer, "  | {}", underline)?;

          // si hay mensaje, lo escribimos al final de la primera linea del span
          if let Some(msg) = &label.message {
            if cur_line == line_start {
              writeln!(self.writer, "    = {}", msg)?;
            }
          }
        }
      }
    }
    Ok(())
  }

  // Funcion responsable de renderizar las notas del diagnostic
  // note: variables must be declared before use
  // help: did you mean `foo`?
  fn render_notes(&mut self, diag: &Diagnostic) -> fmt::Result {
    diag
      .notes()
      .iter()
      .try_for_each(|note| writeln!(self.writer, "{}", note))
  }
}

#[cfg(test)]
mod tests;
