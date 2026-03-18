//! Responsabilidad: definir que tiene que implementar un analisis concreto.
//! Es decir: definir lo que es un problema de dataflow.
//! - tipo de hecho
//! - direccion
//! - transferencia
//! - inicializacion
//! - como combinar predecesores o sucesores

use crate::{
  analysis::{
    Cfg,
    dataflow::{direction::Direction, lattice::Lattice},
  },
  ir::{BlockId, IrModule},
};

pub(crate) trait DataflowProblem: Lattice {
  // type Context; // activar si necesito tablas auxiliares para algun analisis

  /// Define si el analisis es forward o backward.
  fn direction(&self) -> Direction;

  /// Aplica la transferencia de la informacion en el bloque completo.
  /// `fact` representa el hecho del lado por donde entra la informacion al
  /// bloque segun la direccion del analisis:
  /// - `Forward`: el hecho `in`
  /// - `Backward`: el hecho `out`
  fn transfer_block(
    &self,
    block: BlockId,
    fact: &Self::Fact,
    module: &IrModule,
    cfg: &Cfg,
  ) -> Self::Fact;

  /// Hecho inicial de frontera.
  /// En `Forward`, es el `in` del entry block.
  /// En `Backward`, es el `out` del exit block.
  fn boundary_fact(&self) -> Self::Fact;

  /// Hook opcional para personalizar la seed inicial del hecho `in`.
  /// Por defecto, usa `bottom`.
  fn initial_in(&self, _block: BlockId) -> Self::Fact {
    self.bottom()
  }

  /// Hook opcional para personalizar la seed inicial del hecho `out`.
  /// Por defecto, usa `bottom`.
  fn initial_out(&self, _block: BlockId) -> Self::Fact {
    self.bottom()
  }

  // /// Esto lo podemos activar si queremos que el problema sugiera orden de resolucion.
  // fn blocks_to_visit(cfg: &Cfg) -> impl Iterator<Item = BlockId>;
  // se lo dejo al solver, por eso lo comento
}
