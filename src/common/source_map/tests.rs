use crate::common::source_map::SourceMap;
use proptest::{prop_assert, prop_assert_eq, proptest};

#[test]
fn empty_source_test() {
  let source_map = SourceMap::new("", "main.lolo");
  assert_eq!((1, 1), source_map.offset_to_line_column(0));
}

#[test]
fn single_character_source_test() {
  let source_map = SourceMap::new("a", "main.lolo");
  assert_eq!((1, 1), source_map.offset_to_line_column(0));
  assert_eq!((1, 2), source_map.offset_to_line_column(1));
}

#[test]
fn multiple_characters_without_newline_test() {
  let source_map = SourceMap::new("abcdef", "main.lolo");
  assert_eq!((1, 1), source_map.offset_to_line_column(0));
  assert_eq!((1, 4), source_map.offset_to_line_column(3));
  assert_eq!((1, 7), source_map.offset_to_line_column(6));
}

#[test]
fn offset_before_newline_test() {
  let source_map = SourceMap::new("abc\ndef", "main.lolo");
  assert_eq!((1, 3), source_map.offset_to_line_column(2));
}

#[test]
fn offset_in_newline_test() {
  let source_map = SourceMap::new("abc\ndef", "main.lolo");
  assert_eq!((1, 4), source_map.offset_to_line_column(3));
}

#[test]
fn offset_after_newline_test() {
  let source_map = SourceMap::new("abc\ndef", "main.lolo");
  assert_eq!((2, 1), source_map.offset_to_line_column(4));
  assert_eq!((2, 2), source_map.offset_to_line_column(5));
}

#[test]
fn multiple_newlines_test() {
  let source_map = SourceMap::new("a\nb\nc", "main.lolo");
  assert_eq!((1, 1), source_map.offset_to_line_column(0));
  assert_eq!((1, 2), source_map.offset_to_line_column(1));
  assert_eq!((2, 1), source_map.offset_to_line_column(2));
  assert_eq!((2, 2), source_map.offset_to_line_column(3));
  assert_eq!((3, 1), source_map.offset_to_line_column(4));
  assert_eq!((3, 2), source_map.offset_to_line_column(5));
}

#[test]
fn newline_at_end_test() {
  let source_map = SourceMap::new("abc\n", "main.lolo");
  assert_eq!((1, 3), source_map.offset_to_line_column(2));
  assert_eq!((1, 4), source_map.offset_to_line_column(3));
  assert_eq!((2, 1), source_map.offset_to_line_column(4));
}

#[test]
fn two_consecutive_newlines_test() {
  let source_map = SourceMap::new("abc\n\ndef", "main.lolo");
  assert_eq!((1, 4), source_map.offset_to_line_column(3));
  assert_eq!((2, 1), source_map.offset_to_line_column(4));
  assert_eq!((3, 1), source_map.offset_to_line_column(5));
}

#[test]
fn span_in_single_line_test() {
  let source_map = SourceMap::new("abc\ndef", "main.lolo");
  assert_eq!((1, 2, 1, 3), source_map.span_to_line_column(&(1..2)));
}

#[test]
fn span_in_multiple_lines_test() {
  let source_map = SourceMap::new("abc\ndef", "main.lolo");
  assert_eq!((1, 3, 2, 1), source_map.span_to_line_column(&(2..4)));
}

#[test]
fn complete_span_test() {
  let source = "abc\ndef";
  let source_map = SourceMap::new(source, "main.lolo");
  assert_eq!(
    (1, 1, 2, 4),
    source_map.span_to_line_column(&(0..source.len()))
  );
}

#[test]
fn get_nth_line_test() {
  let source_map = SourceMap::new("abc\ndef", "main.lolo");
  assert_eq!((0..3), source_map.get_nth_line(1).unwrap());
  assert_eq!((4..7), source_map.get_nth_line(2).unwrap());
}

#[test]
fn empty_source_has_no_lines() {
  let sm = SourceMap::new("", "main.lolo");
  assert_eq!(None, sm.get_nth_line(1));
}

#[test]
fn single_line_no_newline() {
  let sm = SourceMap::new("abc", "main.lolo");
  // assert_eq!(Some(0..3), sm.get_nth_line(1));
  assert_eq!(None, sm.get_nth_line(2));
}

#[test]
fn trailing_newline_creates_empty_last_line() {
  let sm = SourceMap::new("abc\n", "main.lolo");
  assert_eq!(Some(0..3), sm.get_nth_line(1));
  assert_eq!(Some(4..4), sm.get_nth_line(2)); // línea vacía
  assert_eq!(None, sm.get_nth_line(3));
}

#[test]
fn multiple_lines_basic() {
  let sm = SourceMap::new("abc\ndef\nghi", "main.lolo");
  assert_eq!(Some(0..3), sm.get_nth_line(1));
  assert_eq!(Some(4..7), sm.get_nth_line(2));
  assert_eq!(Some(8..11), sm.get_nth_line(3));
}

#[test]
fn consecutive_newlines_generate_empty_lines() {
  let sm = SourceMap::new("abc\n\n\ndef", "main.lolo");
  assert_eq!(Some(0..3), sm.get_nth_line(1));
  assert_eq!(Some(4..4), sm.get_nth_line(2)); // vacia
  assert_eq!(Some(5..5), sm.get_nth_line(3)); // vacia
  assert_eq!(Some(6..9), sm.get_nth_line(4));
}

#[test]
fn first_line_empty() {
  let sm = SourceMap::new("\nabc", "main.lolo");
  assert_eq!(Some(0..0), sm.get_nth_line(1));
  assert_eq!(Some(1..4), sm.get_nth_line(2));
}

#[test]
fn requesting_invalid_line_returns_none() {
  let sm = SourceMap::new("abc", "main.lolo");
  assert_eq!(None, sm.get_nth_line(0));
  assert_eq!(None, sm.get_nth_line(999));
}

// monotonia de lineas
proptest! {
  #[test]
  fn line_is_monotonic_test(source in ".*", a in 0usize..1000, b in 0usize..1000) {
    let sm = SourceMap::new(&source, "main.lolo");
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
    let sm = SourceMap::new(&source, "main.lolo");
    let offset = offset.min(source.len());
    let (line, col) = sm.offset_to_line_column(offset);
    prop_assert!(line >= 1 && col >= 1);
  }
}

// si no se cruzan lineas: offset + 1 -> columna + 1
proptest! {
  #[test]
  fn column_progresses_correctly(source in ".*", offset in 0usize..1000) {
    let sm = SourceMap::new(&source, "main.lolo");
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

// para cualquier string s, si iteramos todas las líneas con get_nth_line,
// extraemos cada slice, insertamos \n entre ellas... entonces deberiamos obtener s
proptest! {
    #[test]
    fn reconstructing_source_from_lines_is_identity(input in ".*") {
        let sm = SourceMap::new(&input, "main.lolo");
        let mut reconstructed = String::new();
        let mut line_idx = 1;
        while let Some(span) = sm.get_nth_line(line_idx) {
            let line = &input[span.clone()];
            reconstructed.push_str(line);
            if span.end < input.len() && input.as_bytes()[span.end] == b'\n' {
                reconstructed.push('\n');
            }
            line_idx += 1;
        }
        prop_assert_eq!(reconstructed, input);
    }
}

// los spans estan ordenados y no se superponen, ademas de cubrir el source de forma consistente
proptest! {
  #[test]
  fn line_spans_are_sorted_and_non_overlapping(input in ".*") {
    let sm = SourceMap::new(&input, "main.lolo");
    let mut spans = Vec::new();
    let mut line_idx = 1;
    while let Some(span) = sm.get_nth_line(line_idx) {
      spans.push(span);
      line_idx += 1;
    }
    for window in spans.windows(2) {
      let (a, b) = (&window[0], &window[1]);
      // spans validos
      prop_assert!(a.start <= a.end && b.start <= b.end);
      // spans ordenados
      prop_assert!(b.start >= a.start);
      // spans sin overlap
      prop_assert!(b.start >= a.end);
    }
    // todos los spans estan dentro del source
    for span in spans {
      prop_assert!(span.start <= input.len());
      prop_assert!(span.end <= input.len());
    }
  }
}
