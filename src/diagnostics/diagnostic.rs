// Modela un error completo (semanticamente). Conceptualmente contiene todo lo que el compilador desea comunicar:
// - severity (error / warning / note)
// - mensaje principal
// - span primario
// - labels secundarios

// busco que se vea algo asi. esto funcionaria para cualquier modulo
// error: mensaje principal
//   --> archivo:linea:columna
//    |
//   3 |  let x = add 1 true;
//    |                 ^^^ tipo incorrecto
// Flujo conceptual ideal: LexerError → Diagnostic → Renderer → Usuario
// Idem para TypeError, ParserError, etc

// Diseño minimalista: mensaje + linea + subrayado
// Idealmente, un diagnostic debe poder sobrevivir a cualquier renderer

use crate::{
  common::Span,
  diagnostics::{label::Label, severity::Severity},
};

type Note = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
  /// Severidad del error.
  severity: Severity,
  /// Que salio mal.
  msg: String,
  /// Donde esta el problema principal (si estuviera en el codigo).
  primary_span: Option<Span>,
  /// Informacion adicional del error.
  labels: Vec<Label>,
  /// Notas adicionales (para no sobrecargar las labels).
  notes: Vec<Note>,
}

impl Diagnostic {
  // Constructores limpios, y voy agregando contexto progresivamente
  pub(crate) fn error(error_msg: String) -> Self {
    Self {
      severity: Severity::Error,
      msg: error_msg,
      primary_span: None,
      labels: vec![],
      notes: vec![],
    }
  }

  #[cfg(test)]
  pub(crate) fn warning(warning_msg: String) -> Self {
    Self {
      severity: Severity::Warning,
      msg: warning_msg,
      primary_span: None,
      labels: vec![],
      notes: vec![],
    }
  }

  #[cfg(test)]
  pub(crate) fn note(note_msg: String) -> Self {
    Self {
      severity: Severity::Note,
      msg: note_msg,
      primary_span: None,
      labels: vec![],
      notes: vec![],
    }
  }

  #[cfg(test)]
  pub(crate) fn help(help_msg: String) -> Self {
    Self {
      severity: Severity::Help,
      msg: help_msg,
      primary_span: None,
      labels: vec![],
      notes: vec![],
    }
  }

  // Agregar contexto progresivamente
  pub(crate) fn with_span(mut self, span: Span) -> Self {
    self.primary_span = Some(span);
    self
  }

  pub(crate) fn with_label(mut self, label: Label) -> Self {
    self.labels.push(label);
    self
  }

  #[allow(dead_code)]
  pub(crate) fn with_note(mut self, note: Note) -> Self {
    self.notes.push(note);
    self
  }

  // Primary span helper: va a ser util para el renderer
  pub(crate) fn primary_span(&self) -> Option<&Span> {
    self.primary_span.as_ref()
  }
  
  pub(crate) fn severity(&self) -> Severity {
    self.severity
  }

  pub fn msg(&self) -> &str {
    &self.msg
  }

  
  pub(crate) fn labels(&self) -> &[Label] {
    &self.labels
  }

  pub(crate) fn notes(&self) -> &[String] {
    &self.notes
  }

  // Severity helpers (para simplificar logica posterior)
  // fn is_error(&self) -> bool {
  //   matches!(self.severity, Severity::Error)
  // }

  // fn is_warning(&self) -> bool {
  //   matches!(self.severity, Severity::Warning)
  // }

  // fn is_help(&self) -> bool {
  //   matches!(self.severity, Severity::Help)
  // }

  // fn is_note(&self) -> bool {
  //   matches!(self.severity, Severity::Note)
  // }
}

pub(crate) trait Diagnosable {
  fn to_diagnostic(&self) -> Diagnostic;
}
