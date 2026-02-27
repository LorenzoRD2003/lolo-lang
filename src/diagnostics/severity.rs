use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Severity {
  Error,
  Warning,
  Note,
  Help,
}

impl fmt::Display for Severity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Error => write!(f, "error"),
      Self::Warning => write!(f, "warning"),
      Self::Note => write!(f, "note"),
      Self::Help => write!(f, "help"),
    }
  }
}
