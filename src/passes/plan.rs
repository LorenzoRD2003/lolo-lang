use std::iter::repeat_n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PassId {
  Uce,
  Dce,
}

impl PassId {
  #[allow(dead_code)]
  pub(crate) fn name(self) -> &'static str {
    match self {
      Self::Uce => "uce",
      Self::Dce => "dce",
    }
  }

  fn parse(token: &str) -> Option<Self> {
    match token.trim().to_ascii_lowercase().as_str() {
      "uce" => Some(Self::Uce),
      "dce" => Some(Self::Dce),
      _ => None,
    }
  }

  /// Determina si el Pass invalida el CFG. Esto es util porque indica si
  /// hay que recomputar el CFG.
  pub(crate) fn invalidates_cfg(self) -> bool {
    match self {
      // UCE cambia estructura del CFG.
      Self::Uce => true,
      // DCE solo elimina instrucciones/phis, no altera control flow.
      Self::Dce => false,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PassSpec {
  pub(crate) id: PassId,
  pub(crate) repeat: usize,
}

impl PassSpec {
  pub(crate) fn new(id: PassId, repeat: usize) -> Self {
    assert!(repeat > 0, "repeat debe ser > 0");
    Self { id, repeat }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PassPlan {
  specs: Vec<PassSpec>,
}

impl Default for PassPlan {
  fn default() -> Self {
    Self {
      specs: vec![PassSpec::new(PassId::Uce, 1), PassSpec::new(PassId::Dce, 1)],
    }
  }
}

impl PassPlan {
  #[allow(dead_code)]
  pub(crate) fn from_specs(specs: Vec<PassSpec>) -> Self {
    Self { specs }
  }

  pub(crate) fn parse(spec: &str) -> Result<Self, String> {
    if spec.trim().is_empty() {
      return Err("--passes no puede ser vacio".into());
    }

    let mut specs = Vec::new();
    for raw_item in spec.split(',') {
      let item = raw_item.trim();
      if item.is_empty() {
        return Err(format!("item vacio en --passes: '{spec}'"));
      }

      let (name, repeat) = match item.split_once('*') {
        Some((name, repeat_str)) => {
          if repeat_str.contains('*') {
            return Err(format!("formato invalido en --passes: '{item}'"));
          }
          let repeat = repeat_str.trim().parse::<usize>().map_err(|_| {
            format!("repeticion invalida en --passes: '{repeat_str}' (item '{item}')")
          })?;
          if repeat == 0 {
            return Err(format!("repeticion debe ser > 0 en --passes: '{item}'"));
          }
          (name.trim(), repeat)
        }
        None => (item, 1),
      };

      let id = PassId::parse(name)
        .ok_or_else(|| format!("pass desconocida '{name}'. soportadas: uce,dce"))?;
      specs.push(PassSpec::new(id, repeat));
    }

    Ok(Self { specs })
  }

  #[allow(dead_code)]
  pub(crate) fn specs(&self) -> &[PassSpec] {
    &self.specs
  }

  pub(crate) fn expanded_passes(&self) -> impl Iterator<Item = PassId> + '_ {
    self
      .specs
      .iter()
      .flat_map(|spec| repeat_n(spec.id, spec.repeat))
  }
}

#[cfg(test)]
mod tests {
  use super::{PassId, PassPlan, PassSpec};

  #[test]
  fn default_plan_is_uce_then_dce() {
    let plan = PassPlan::default();
    assert_eq!(
      plan.specs(),
      &[PassSpec::new(PassId::Uce, 1), PassSpec::new(PassId::Dce, 1)]
    );
  }

  #[test]
  fn parse_single_and_repeated_passes() {
    let plan = PassPlan::parse("uce,dce*2,uce").expect("parse valido");
    assert_eq!(
      plan.specs(),
      &[
        PassSpec::new(PassId::Uce, 1),
        PassSpec::new(PassId::Dce, 2),
        PassSpec::new(PassId::Uce, 1),
      ]
    );

    let expanded: Vec<_> = plan.expanded_passes().collect();
    assert_eq!(
      expanded,
      vec![PassId::Uce, PassId::Dce, PassId::Dce, PassId::Uce]
    );
  }

  #[test]
  fn parse_is_case_insensitive_and_trims_whitespace() {
    let plan = PassPlan::parse(" UCE , dCe * 3 ").expect("parse valido");
    assert_eq!(
      plan.specs(),
      &[PassSpec::new(PassId::Uce, 1), PassSpec::new(PassId::Dce, 3)]
    );
  }

  #[test]
  fn parse_rejects_empty_spec() {
    let err = PassPlan::parse("   ").expect_err("debe fallar");
    assert!(err.contains("no puede ser vacio"));
  }

  #[test]
  fn parse_rejects_unknown_pass() {
    let err = PassPlan::parse("foo").expect_err("debe fallar");
    assert!(err.contains("pass desconocida"));
  }

  #[test]
  fn parse_rejects_zero_repeat() {
    let err = PassPlan::parse("dce*0").expect_err("debe fallar");
    assert!(err.contains("> 0"));
  }

  #[test]
  fn parse_rejects_invalid_repeat() {
    let err = PassPlan::parse("dce*abc").expect_err("debe fallar");
    assert!(err.contains("repeticion invalida"));
  }

  #[test]
  fn parse_rejects_malformed_item() {
    let err = PassPlan::parse("uce**2").expect_err("debe fallar");
    assert!(err.contains("formato invalido"));
  }

  #[test]
  fn pass_id_metadata_is_correct() {
    assert_eq!(PassId::Uce.name(), "uce");
    assert_eq!(PassId::Dce.name(), "dce");
    assert!(PassId::Uce.invalidates_cfg());
    assert!(!PassId::Dce.invalidates_cfg());
  }
}
