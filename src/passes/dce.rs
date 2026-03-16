use std::collections::VecDeque;

use crate::{
  analysis::Cfg,
  ir::{InstId, InstKind, IrModule, ValueId},
  passes::{IrPass, PassContext, PassStats},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DceStats {
  pub(crate) removed_phis: usize,
  pub(crate) removed_insts: usize,
}

pub(crate) struct DcePass;

impl DcePass {
  fn run(module: &mut IrModule, cfg: &Cfg) -> DceStats {
    // Instrucciones vivas segun DCE
    let mut live_insts = vec![false; module.inst_count()];
    // Cola de instrucciones pendientes de procesar: es un backward analysis
    let mut worklist: VecDeque<InstId> = VecDeque::new();

    let value_def = Self::build_value_def(module);
    Self::seed_roots(module, cfg, &mut live_insts, &mut worklist);
    Self::propagate(module, &value_def, &mut live_insts, &mut worklist);
    Self::sweep(module, cfg, &live_insts)
  }

  /// Construye un mapa ValueId -> InstId. En que instruccion fue definido cada valor.
  fn build_value_def(module: &IrModule) -> Vec<Option<InstId>> {
    let Some(max_value_id) = Self::find_max_value_id(module) else {
      return vec![];
    };
    let mut value_def = vec![None; max_value_id.0 + 1];

    // Guarda en la instruccion asociada al valor, para cada instruccion que define_valor
    for i in 0..module.inst_count() {
      let inst_id = InstId(i);
      if let Some(result) = module.inst(inst_id).result {
        value_def[result.0] = Some(inst_id);
      }
    }

    value_def
  }

  /// Mete los nodos iniciales "raices" en la worklist y los marca como vivos
  fn seed_roots(
    module: &IrModule,
    cfg: &Cfg,
    live_insts: &mut [bool],
    worklist: &mut VecDeque<InstId>,
  ) {
    for block_id in cfg.reachable_blocks() {
      let block = module.block(block_id);

      // En IR valida todo bloque alcanzable tiene terminador.
      debug_assert!(block.has_terminator(), "bloque alcanzable sin terminador");
      Self::mark_live(block.terminator(), live_insts, worklist);

      // Cualquier instruccion con side effects observable tambien es raiz.
      for &inst_id in block.insts() {
        if Self::is_side_effecting(&module.inst(inst_id).kind) {
          Self::mark_live(inst_id, live_insts, worklist);
        }
      }
    }
  }

  /// Marca como viva una instruccion. La agenda para futura propagacion.
  /// Debe recorrer sus operandos, y para cada uno,
  /// buscar cuando la instruccion fue definida (`value_def`) y encolarla.
  fn mark_live(inst_id: InstId, live_insts: &mut [bool], worklist: &mut VecDeque<InstId>) {
    if live_insts[inst_id.0] {
      return;
    }
    live_insts[inst_id.0] = true;
    worklist.push_back(inst_id);
  }

  /// Calcula la clausura del conjunto de instrucciones vivas.
  fn propagate(
    module: &IrModule,
    value_def: &[Option<InstId>],
    live_insts: &mut [bool],
    worklist: &mut VecDeque<InstId>,
  ) {
    while let Some(next) = worklist.pop_front() {
      let inst = module.inst(next);
      inst.kind.for_each_operand(|operand| {
        // Busco la instruccion donde fue definido el operando, y la encolo
        if let Some(def_inst) = value_def[operand.0] {
          Self::mark_live(def_inst, live_insts, worklist)
        }
      });
    }
  }

  /// Remueve las instrucciones muertas para cada bloque alcanzable.
  fn sweep(module: &mut IrModule, cfg: &Cfg, live_insts: &[bool]) -> DceStats {
    let (mut removed_phis, mut removed_insts) = (0, 0);

    for bid in cfg.reachable_blocks() {
      let block = module.block_mut(bid);
      let (before_phis, before_insts) = (block.phis().len(), block.insts().len());

      block.retain_phis(|inst_id| live_insts[inst_id.0]);
      block.retain_insts(|inst_id| live_insts[inst_id.0]);

      removed_phis += before_phis - block.phis().len();
      removed_insts += before_insts - block.insts().len();
    }

    DceStats {
      removed_phis,
      removed_insts,
    }
  }

  /// Indica si la instruccion tiene efectos secundarios
  /// Por ahora solo tenemos `InstKind::Print`
  fn is_side_effecting(kind: &InstKind) -> bool {
    matches!(kind, InstKind::Print(_))
  }

  /// Devuelve el `ValueId` maximo observado entre resultados y operandos de instrucciones
  fn find_max_value_id(module: &IrModule) -> Option<ValueId> {
    let mut max_value_id: Option<usize> = None;
    for i in 0..module.inst_count() {
      let inst_id = InstId(i);
      let inst = module.inst(inst_id);

      if let Some(result) = inst.result {
        max_value_id = Some(max_value_id.map_or(result.0, |m| m.max(result.0)));
      }
      inst.kind.for_each_operand(|operand| {
        max_value_id = Some(max_value_id.map_or(operand.0, |m| m.max(operand.0)));
      });
    }
    max_value_id.map(ValueId)
  }
}

impl IrPass for DcePass {
  fn name(&self) -> &'static str {
    "dce"
  }

  fn run(&self, module: &mut IrModule, ctx: &PassContext) -> PassStats {
    let stats = Self::run(module, ctx.cfg());
    PassStats::Dce(stats)
  }
}

#[cfg(test)]
mod tests;
