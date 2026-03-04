// El ejecutor es el que EJECUTA las fases, y puede hacerlo en paralelo si se cumplen los requisitos.
// TODO: ver como adaptar el &mut diagnostics para que no falle

// pseudocodigo
// while no_todas_completadas {
//     let ready = ready_phases(completed);
//     ejecutar ready en paralelo;
//     for fase in ready {
//         mark_completed(fase);
//     }
// }

use rayon::prelude::*;

use crate::{
  ast::{Ast, Program},
  semantic::{context::SemanticContext, phase::PhaseOutput, phase_graph::PhaseGraph},
};

pub(crate) struct Executor;

impl Executor {
  pub(crate) fn execute<'a>(
    ast: &'a Ast,
    program: &Program,
    graph: &mut PhaseGraph<'a>,
    ctx: &mut SemanticContext,
  ) {
    while !graph.all_phases_completed() {
      let ready = graph.ready_phases();

      let outputs: Vec<(&str, PhaseOutput)> = ready
        .par_iter()
        .map(|node| {
          let output = node.phase.run(ast, program, ctx);
          (node.phase.name(), output)
        })
        .collect();

      for (name, output) in outputs {
        ctx.apply_phase_output(output);
        graph.mark_phase_completed(name);
      }
    }
  }
}
