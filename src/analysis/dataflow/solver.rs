//! Implementacion del algoritmo iterativo de convergencia al punto fijo.
//! Este archivo no deberia saber nada del analisis concreto.

use crate::{
  analysis::{
    Cfg,
    dataflow::{
      direction::Direction, problem::DataflowProblem, result::DataflowResult, worklist::Worklist,
    },
  },
  ir::{BlockId, IrModule},
};

pub(crate) struct DataflowSolver;

impl DataflowSolver {
  /// Resuelve un problema de dataflow sobre el CFG del modulo hasta alcanzar
  /// un punto fijo, y devuelve los hechos `in` y `out` por bloque.
  pub(crate) fn solve<DFP>(problem: &DFP, module: &IrModule, cfg: &Cfg) -> DataflowResult<DFP::Fact>
  where
    DFP: DataflowProblem,
  {
    let mut result = Self::initialize(problem, cfg);
    Self::run_worklist(problem, module, cfg, &mut result);
    result
  }

  /// Construye el resultado inicial del analisis.
  /// - todos los bloques arrancan en `bottom`
  /// - los bloques de frontera arrancan con `boundary_fact`
  fn initialize<DFP>(problem: &DFP, cfg: &Cfg) -> DataflowResult<DFP::Fact>
  where
    DFP: DataflowProblem,
  {
    let mut result = DataflowResult::new(cfg.block_count(), problem.bottom());
    for block in cfg.blocks() {
      let mut in_fact = problem.initial_in(block);
      let mut out_fact = problem.initial_out(block);

      if Self::should_seed_boundary(problem.direction(), block, cfg) {
        match problem.direction() {
          Direction::Forward => in_fact = problem.boundary_fact(),
          Direction::Backward => out_fact = problem.boundary_fact(),
        }
      }

      result.set_in_fact(block, in_fact);
      result.set_out_fact(block, out_fact);
    }
    result
  }

  /// Ejecuta la iteracion principal del algoritmo hasta que no haya cambios
  /// en los hechos asociados a ningun bloque.
  fn run_worklist<P>(
    problem: &P,
    module: &IrModule,
    cfg: &Cfg,
    result: &mut DataflowResult<P::Fact>,
  ) where
    P: DataflowProblem,
  {
    let mut worklist = Worklist::new(cfg.block_count());
    worklist.extend(cfg.reachable_blocks());

    while let Some(block) = worklist.pop() {
      let changed = match problem.direction() {
        Direction::Forward => {
          let new_in = if Self::should_seed_boundary(Direction::Forward, block, cfg) {
            result.in_fact(block).clone()
          } else {
            Self::compute_merged_fact(problem, block, cfg, result)
          };
          let new_out = Self::compute_transferred_fact(problem, block, &new_in, module, cfg);

          let in_changed = result.in_fact(block) != &new_in;
          let out_changed = result.out_fact(block) != &new_out;

          if in_changed {
            result.set_in_fact(block, new_in);
          }
          if out_changed {
            result.set_out_fact(block, new_out);
          }

          in_changed || out_changed
        }
        Direction::Backward => {
          let new_out = if Self::should_seed_boundary(Direction::Backward, block, cfg) {
            result.out_fact(block).clone()
          } else {
            Self::compute_merged_fact(problem, block, cfg, result)
          };
          let new_in = Self::compute_transferred_fact(problem, block, &new_out, module, cfg);

          let out_changed = result.out_fact(block) != &new_out;
          let in_changed = result.in_fact(block) != &new_in;

          if out_changed {
            result.set_out_fact(block, new_out);
          }
          if in_changed {
            result.set_in_fact(block, new_in);
          }

          in_changed || out_changed
        }
      };

      if changed {
        match problem.direction() {
          Direction::Forward => worklist.extend(cfg.successors(block).iter().copied()),
          Direction::Backward => worklist.extend(cfg.predecessors(block).iter().copied()),
        }
      }
    }
  }

  /// Computa el hecho del lado por donde entra la informacion a `block`,
  /// combinando los hechos de los vecinos relevantes segun la direccion del
  /// analisis.
  fn compute_merged_fact<P>(
    problem: &P,
    block: BlockId,
    cfg: &Cfg,
    result: &DataflowResult<P::Fact>,
  ) -> P::Fact
  where
    P: DataflowProblem,
  {
    match problem.direction() {
      Direction::Forward => {
        let preds = cfg
          .predecessors(block)
          .iter()
          .map(|&pred| result.out_fact(pred));
        problem.meet_all(preds)
      }
      Direction::Backward => {
        let succs = cfg
          .successors(block)
          .iter()
          .map(|&succ| result.in_fact(succ));
        problem.meet_all(succs)
      }
    }
  }

  /// Computa el hecho del lado opuesto del bloque aplicando la transferencia
  /// del problema al hecho de entrada segun la direccion del analisis.
  fn compute_transferred_fact<P>(
    problem: &P,
    block: BlockId,
    fact: &P::Fact,
    module: &IrModule,
    cfg: &Cfg,
  ) -> P::Fact
  where
    P: DataflowProblem,
  {
    problem.transfer_block(block, fact, module, cfg)
  }

  /// Indica si `block` pertenece a la frontera del analisis y por lo tanto
  /// debe inicializarse con `boundary_fact`.
  ///
  /// Convencion actual:
  /// - `Forward`: la frontera es el bloque de entrada
  /// - `Backward`: la frontera son los bloques sin sucesores
  fn should_seed_boundary(direction: Direction, block: BlockId, cfg: &Cfg) -> bool {
    match direction {
      Direction::Forward => block == cfg.entry(),
      Direction::Backward => cfg.successors(block).is_empty(),
    }
  }
}
