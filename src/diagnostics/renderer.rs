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
  common::{source_map::SourceMap, span::Span},
  diagnostics::diagnostic::Diagnostic,
};
use std::fmt;

pub struct Renderer<'a, W: fmt::Write> {
  // SourceMap, para poder traducir los spans
  source_map: &'a SourceMap<'a>,
  // output target, para indicar a donde van a renderizarse los mensajes
  writer: W,
  // configuracion visual (colores, ascii/unicode, compact/verbose) -> por ahora no
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
    writeln!(self.writer, "{}: {}", diag.severity, diag.msg)
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

  // Funcion responsable de renderizar las notas del diagnostic
  // note: variables must be declared before use
  // help: did you mean `foo`?
  fn render_notes(&mut self, diag: &Diagnostic) -> fmt::Result {
    diag
      .notes
      .iter()
      .try_for_each(|note| writeln!(self.writer, "{}", note))
  }
}

pub mod tests {
  use super::*;
  use proptest::prelude::*;

  #[test]
  fn renders_header() {
    let diag = Diagnostic::error("boom".into());
    let mut out = String::new();
    let sm = SourceMap::new("", "main.lolo");
    let mut renderer = Renderer::new(&sm, &mut out);
    renderer.render_header(&diag).unwrap();
    assert_eq!(out, "error: boom\n");
  }

  #[test]
  fn renders_location_with_span() {
    let sm = SourceMap::new("abc", "main.lolo");
    let diag = Diagnostic::warning("warning".into()).with_span(1..2);

    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);
    renderer.render_location(&diag).unwrap();
    assert_eq!(out, " --> main.lolo:1:2\n");
  }

  #[test]
  fn renders_location_without_span() {
    let sm = SourceMap::new("abc", "main.lolo");
    let diag = Diagnostic::help("help".into());

    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);
    renderer.render_location(&diag).unwrap();
    assert_eq!(out, " --> main.lolo:unknown location\n");
  }

  #[test]
  fn renders_single_line_diagnostic() {
    let source = "let x = add 1 true;";
    let sm = SourceMap::new(source, "main.lolo");

    let diag = Diagnostic::error("type mismatch".into()).with_span(14..18); // "true"
    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);

    renderer.render(&diag).unwrap();
    let expected = "error: type mismatch\n --> main.lolo:1:15\n  |\n1 | let x = add 1 true;\n  |               ^^^^\n";
    assert_eq!(out, expected);
  }

  #[test]
  fn renders_multiline_snippet() {
    let source = "aaa\nbbb\nccc";
    let sm = SourceMap::new(source, "main.lolo");

    let diag = Diagnostic::note("note: aguante boca".into()).with_span(2..8);
    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);

    renderer.render_code_snippet(&diag).unwrap();
    let expected = "  |\n1 | aaa\n  |   ^\n2 | bbb\n  | ^^^\n3 | ccc\n  | ^\n";
    assert_eq!(out, expected);
  }

  #[test]
  fn renders_span_at_line_start() {
    let source = "abcdef";
    let sm = SourceMap::new(source, "main.lolo");

    let diag = Diagnostic::error("boom".into()).with_span(0..3);
    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);

    renderer.render_code_snippet(&diag).unwrap();
    assert!(out.contains("^^^"));
  }

  #[test]
  fn renders_span_at_line_end() {
    let source = "abcdef";
    let sm = SourceMap::new(source, "main.lolo");

    let diag = Diagnostic::error("boom".into()).with_span(3..6);
    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);

    renderer.render_code_snippet(&diag).unwrap();
    assert!(out.contains("^^^"));
  }

  #[test]
  fn renders_notes() {
    let sm = SourceMap::new("abc", "main.lolo");

    let diag = Diagnostic::error("boom".into())
      .with_note("note: something".into())
      .with_note("help: do X".into());
    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);

    renderer.render_notes(&diag).unwrap();
    let expected = "note: something\n\
      help: do X\n";
    assert_eq!(out, expected);
  }

  proptest! {
    #[test]
    fn renderer_never_panics(
      input in ".*",
      start in 0usize..100,
      len in 0usize..50
    ) {
      let sm = SourceMap::new(&input, "main.lolo");
      let safe_start = start.min(input.len());
      let safe_end = (safe_start + len).min(input.len());
      let diag = Diagnostic::error("boom".into()).with_span(safe_start..safe_end);

      let mut out = String::new();
      let mut renderer = Renderer::new(&sm, &mut out);
      let _ = renderer.render(&diag);
    }
  }

  // este proptest deberia detectar el bug del span vacio visual
  proptest! {
    #[test]
    fn underline_is_never_empty_when_span_visible(
      input in ".+",
      start in 0usize..100,
      len in 1usize..50
    ) {
      let sm = SourceMap::new(&input, "main.lolo");
      let safe_start = start.min(input.len() - 1);
      let safe_end = (safe_start + len).min(input.len());
      let diag = Diagnostic::error("boom".into()).with_span(safe_start..safe_end);
      let mut out = String::new();
      let mut renderer = Renderer::new(&sm, &mut out);
      renderer.render_code_snippet(&diag).unwrap();

      // Si renderizamos el snippet debe haber al menos un ^
      prop_assert!(out.contains("^"));
    }
  }

  // offset mapping consistente con slicing
  proptest! {
    #[test]
    fn span_render_matches_source_length(
      input in ".*",
      start in 0usize..100,
      len in 0usize..50
    ) {
      let sm = SourceMap::new(&input, "main.lolo");
      let safe_start = start.min(input.len());
      let safe_end = (safe_start + len).min(input.len());
      let diag = Diagnostic::error("boom".into()).with_span(safe_start..safe_end);
      let mut out = String::new();
      let mut renderer = Renderer::new(&sm, &mut out);
      let _ = renderer.render_code_snippet(&diag);

      // Nunca deberia renderizar caracteres fuera del source
      prop_assert!(input.contains("\0") || !out.contains("\0"));
    }
  }
}
