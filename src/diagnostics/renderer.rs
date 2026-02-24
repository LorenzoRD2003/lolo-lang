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
  diagnostics::{diagnostic::Diagnostic, label::LabelStyle},
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

  /// Renderiza los labels secundarios (y primarios opcionales) de un Diagnostic
  fn render_labels(&mut self, diag: &Diagnostic) -> fmt::Result {
    for label in &diag.labels {
      let (line_start, column_start, line_end, column_end) =
        self.source_map.span_to_line_column(&label.span);

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
      .notes
      .iter()
      .try_for_each(|note| writeln!(self.writer, "{}", note))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::diagnostics::label::Label;
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
  fn renders_primary_and_secondary_labels() {
    let source = "abcde\nfghij\nklmno";
    let sm = SourceMap::new(source, "main.lolo");

    let diag = Diagnostic::warning("warning".into())
      .with_label(Label::primary(0..2, Some("primary here".into())))
      .with_label(Label::secondary(5..7, Some("secondary here".into())));

    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);
    renderer.render_labels(&diag).unwrap();

    assert!(out.contains("^")); // primary
    assert!(out.contains("~")); // secondary
    assert!(out.contains("primary here"));
    assert!(out.contains("secondary here"));
  }

  #[test]
  fn renders_multiline_label() {
    let source = "1\n2\n3\n432\n5\n";
    let sm = SourceMap::new(source, "file.lolo");

    let diag = Diagnostic::error("oops".into())
      .with_label(Label::secondary(0..12, Some("multiline epic label".into()))); // abarca line1 + line2

    let mut out = String::new();
    let mut renderer = Renderer::new(&sm, &mut out);
    renderer.render_labels(&diag).unwrap();

    // debería contener varias líneas de '~'
    let dash_count = out.chars().filter(|&c| c == '~').count();
    dbg!(dash_count);
    assert!(dash_count >= 6); // al menos un dash por cada linea del span
    assert!(out.contains("multiline"));
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
    fn renderer_never_panics(bytes in proptest::collection::vec(0u8..=127u8, 0..100),
      start in 0usize..100,
      len in 1usize..50
    ) {
      let input = String::from_utf8(bytes).unwrap();
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
    fn underline_is_never_empty_when_span_visible(bytes in proptest::collection::vec(0u8..=127u8, 1..100),
      start in 0usize..100,
      len in 1usize..50
    ) {
      let input = String::from_utf8(bytes).unwrap();
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
    fn span_render_matches_source_length(bytes in proptest::collection::vec(0u8..=127u8, 0..100),
      start in 0usize..100,
      len in 1usize..50
    ) {
      let input = String::from_utf8(bytes).unwrap();
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

  // Property test: los labels nunca subrayan fuera del source
  proptest! {
    #[test]
    fn label_subspan_within_source(bytes in proptest::collection::vec(0u8..=127u8, 1..100),
      start in 0usize..100,
      len in 1usize..50
    ) {
      let input = String::from_utf8(bytes).unwrap();
      let sm = SourceMap::new(&input, "file.lolo");
      let safe_start = start.min(input.len() - 1);
      let safe_end = (safe_start + len).min(input.len());
      let diag = Diagnostic::error("prop test".into())
          .with_label(Label::secondary(safe_start..safe_end, Some("label".into())));

      let mut out = String::new();
      let mut renderer = Renderer::new(&sm, &mut out);
      renderer.render_labels(&diag).unwrap();

      // nunca subrayar fuera del input
      let slice = &input[safe_start..safe_end];
      prop_assert!(slice.len() >= 1);
      prop_assert!(out.contains("~"));
    }
  }
}
