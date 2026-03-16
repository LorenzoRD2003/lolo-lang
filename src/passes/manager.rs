use crate::{
  analysis::CfgError,
  ir::IrModule,
  passes::{
    DcePass, IrPass, PassContext, PassStats, UcePass,
    plan::{PassId, PassPlan},
  },
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PassManager;

impl PassManager {
  pub(crate) fn run(
    module: &mut IrModule,
    plan: &PassPlan,
  ) -> Result<Vec<PassStats>, Vec<CfgError>> {
    let mut stats = Vec::new();
    let mut cached_ctx: Option<PassContext> = None;

    for pass_id in plan.expanded_passes() {
      if cached_ctx.is_none() {
        cached_ctx = Some(PassContext::from_module(module)?);
      }

      let ctx = cached_ctx
        .as_ref()
        .expect("siempre hay PassContext antes de ejecutar pass");
      let pass_stat = match pass_id {
        PassId::Uce => UcePass.run(module, ctx),
        PassId::Dce => DcePass.run(module, ctx),
      };
      stats.push(pass_stat);

      if pass_id.invalidates_cfg() {
        cached_ctx = None;
      }
    }

    Ok(stats)
  }
}

#[cfg(test)]
mod tests {
  use super::PassManager;
  use crate::{
    ir::{BlockData, BlockId, InstData, InstId, InstKind, IrModule, IrType},
    passes::{
      PassStats,
      plan::{PassId, PassPlan, PassSpec},
    },
  };

  fn module_with_shift_after_uce() -> IrModule {
    // old blocks: 0(unreachable), 1(entry)->2(return), 2(return)
    // UCE elimina bb0 y remapea bb1->bb0, bb2->bb1.
    // Si DCE usara CFG viejo (reachable {1,2}), accederia bb2 invalido y explotaria.
    let mut m = IrModule::new("m".into(), IrType::Unit);
    for _ in 0..3 {
      m.add_block(BlockData::new_block());
    }

    m.set_entry_block(BlockId(1));

    let t0 = InstId(m.inst_count());
    m.add_inst(InstData::without_result(InstKind::Return { value: None }));
    m.block_mut(BlockId(0)).set_terminator(t0);

    let t1 = InstId(m.inst_count());
    m.add_inst(InstData::without_result(InstKind::Jump {
      target: BlockId(2),
    }));
    m.block_mut(BlockId(1)).set_terminator(t1);

    let t2 = InstId(m.inst_count());
    m.add_inst(InstData::without_result(InstKind::Return { value: None }));
    m.block_mut(BlockId(2)).set_terminator(t2);

    m
  }

  #[test]
  fn run_executes_passes_in_declared_order_and_repetition() {
    let source = r#"
      main {
        let x = 1;
        return x;
      }
    "#;
    let (mut ir, diagnostics) = crate::ir::test_helpers::lower_source(source);
    assert!(diagnostics.is_empty());

    let plan = PassPlan::from_specs(vec![
      PassSpec::new(PassId::Dce, 1),
      PassSpec::new(PassId::Uce, 1),
      PassSpec::new(PassId::Dce, 2),
    ]);

    let stats = PassManager::run(&mut ir, &plan).expect("passes deben correr");
    assert_eq!(stats.len(), 4);
    assert!(matches!(stats[0], PassStats::Dce(_)));
    assert!(matches!(stats[1], PassStats::Uce(_)));
    assert!(matches!(stats[2], PassStats::Dce(_)));
    assert!(matches!(stats[3], PassStats::Dce(_)));
  }

  #[test]
  fn run_rebuilds_cfg_after_uce_invalidation() {
    let mut ir = module_with_shift_after_uce();
    let plan = PassPlan::from_specs(vec![
      PassSpec::new(PassId::Uce, 1),
      PassSpec::new(PassId::Dce, 1),
    ]);

    let stats = PassManager::run(&mut ir, &plan).expect("debe reconstruir CFG luego de UCE");
    assert_eq!(stats.len(), 2);
    assert!(matches!(stats[0], PassStats::Uce(_)));
    assert!(matches!(stats[1], PassStats::Dce(_)));
    assert_eq!(ir.block_count(), 2);
  }

  #[test]
  fn run_returns_empty_stats_for_empty_plan() {
    let source = "main { return 0; }";
    let (mut ir, diagnostics) = crate::ir::test_helpers::lower_source(source);
    assert!(diagnostics.is_empty());

    let plan = PassPlan::from_specs(vec![]);
    let stats = PassManager::run(&mut ir, &plan).expect("plan vacio debe ser valido");
    assert!(stats.is_empty());
  }
}
