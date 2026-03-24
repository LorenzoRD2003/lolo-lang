// Responsabilidad: Dar una API ergonomica para construir IR sin andar manipulando todos los `Vec` a mano.

use rustc_hash::FxHashMap;

use crate::{
  ast::{BinaryOp, UnaryOp},
  common::{IdGenerator, IncrementalIdGenerator},
  ir::{
    block::BlockData,
    ids::{BlockId, InstId, ValueId},
    inst::{InstData, InstKind, PhiInput},
    ir_source_map::IrSourceMap,
    module::IrModule,
    types::IrType,
    value::{IrConstant, ValueData, ValueKind},
  },
};

#[derive(Debug, Clone)]
pub(crate) struct ProgramBuilder {
  program: IrModule,
  /// bloque en el cual el builder esta insertando instrucciones ahora
  current_block: Option<BlockId>,
  inst_id_generator: IncrementalIdGenerator<InstId>,
  block_id_generator: IncrementalIdGenerator<BlockId>,
  value_id_generator: IncrementalIdGenerator<ValueId>,
  /// cache de constantes para evitar duplicados
  const_cache: FxHashMap<IrConstant, ValueId>,
  #[allow(dead_code)]
  /// mapa entre la IR y el codigo fuente
  ir_source_map: IrSourceMap,
}

impl ProgramBuilder {
  pub(crate) fn new(name: String, return_type: IrType) -> Self {
    let program = IrModule::new(name, return_type);
    let mut program_builder = Self {
      current_block: None,
      program,
      inst_id_generator: IncrementalIdGenerator::new(),
      block_id_generator: IncrementalIdGenerator::new(),
      value_id_generator: IncrementalIdGenerator::new(),
      const_cache: FxHashMap::default(),
      ir_source_map: IrSourceMap::new(),
    };

    let entry_block = program_builder.create_block();
    program_builder.program.set_entry_block(entry_block);
    program_builder.switch_to_block(entry_block);

    program_builder
  }

  // ======================
  //   Obtencion de datos
  // ======================

  pub(crate) fn get_value_type(&self, id: ValueId) -> IrType {
    *self.program.value(id).ty()
  }

  // ===========================
  //   Creacion de estructuras
  // ===========================

  pub(crate) fn create_block(&mut self) -> BlockId {
    let block_data = BlockData::new_block();
    self.program.add_block(block_data);
    self.block_id_generator.next_id()
  }

  /// Actualiza el bloque en el cual estamos introduciendo instrucciones ahora
  pub(crate) fn switch_to_block(&mut self, id: BlockId) {
    self.current_block = Some(id)
  }

  /// Devuelve el bloque actual de emision.
  pub(crate) fn current_block_id(&self) -> BlockId {
    self.current_block()
  }

  // pub(crate) fn current_block_has_terminator(&self) -> bool {
  //   self
  //     .current_block
  //     .is_some_and(|bid| self.program.block(bid).has_terminator())
  // }

  // ========================================
  //   instrucciones de creacion de valores
  // ========================================

  pub(crate) fn emit_const(&mut self, c: IrConstant) -> ValueId {
    if let Some(&value_id) = self.const_cache.get(&c) {
      return value_id;
    }

    let value_id = self.value_id_generator.next_id();
    self.program.add_value(c.as_value());

    let inst_data = InstData::with_result(value_id, InstKind::Const(c.clone()));
    self.add_inst_to_program(inst_data);

    self.const_cache.insert(c, value_id);
    value_id
  }

  pub(crate) fn emit_inst_result_value(&mut self, ty: IrType) -> ValueId {
    let value_id = self.value_id_generator.next_id();
    let value_data = ValueData::new(ty, ValueKind::InstResult);
    self.program.add_value(value_data);
    value_id
  }

  pub(crate) fn emit_unary(&mut self, op: UnaryOp, operand: ValueId) -> ValueId {
    let value_id = self.emit_inst_result_value(op.result_type().into());
    let inst_data = InstData::with_result(value_id, InstKind::Unary { op, operand });
    self.add_inst_to_program(inst_data);
    value_id
  }

  pub(crate) fn emit_binary(&mut self, op: BinaryOp, lhs: ValueId, rhs: ValueId) -> ValueId {
    let value_id = self.emit_inst_result_value(op.result_type().into());
    let inst_data = InstData::with_result(value_id, InstKind::Binary { op, lhs, rhs });
    self.add_inst_to_program(inst_data);
    value_id
  }

  pub(crate) fn emit_phi(&mut self, ty: IrType, inputs: Vec<PhiInput>) -> ValueId {
    self.emit_phi_in_block(self.current_block(), ty, inputs)
  }

  /// Emitir phi nodes al comienzo de un bloque
  pub(crate) fn emit_phi_in_block(
    &mut self,
    block: BlockId,
    ty: IrType,
    inputs: Vec<PhiInput>,
  ) -> ValueId {
    let value_id = self.value_id_generator.next_id();
    let inst_id = self.inst_id_generator.next_id();
    self
      .program
      .add_value(ValueData::new(ty, ValueKind::InstResult));

    let inst = InstData::with_result(value_id, InstKind::Phi { inputs });
    self.program.add_inst(inst);
    self.program.block_mut(block).add_phi(inst_id);

    value_id
  }

  // ==============================
  //   Instrucciones terminadoras
  // ==============================

  pub(crate) fn emit_jump(&mut self, target: BlockId) -> InstId {
    let inst_data = InstData::without_result(InstKind::Jump { target });
    self.add_inst_to_program(inst_data)
  }

  pub(crate) fn emit_branch(
    &mut self,
    condition: ValueId,
    if_block: BlockId,
    else_block: BlockId,
  ) -> InstId {
    let inst_data = InstData::without_result(InstKind::Branch {
      condition,
      if_block,
      else_block,
    });
    self.add_inst_to_program(inst_data)
  }

  pub(crate) fn emit_return(&mut self, value: Option<ValueId>) -> InstId {
    let inst_data = InstData::without_result(InstKind::Return { value });
    self.add_inst_to_program(inst_data)
  }

  // =============================
  //   Instrucciones miscelaneas
  // =============================

  pub(crate) fn emit_print(&mut self, value: ValueId) -> InstId {
    let inst_data = InstData::without_result(InstKind::Print(value));
    self.add_inst_to_program(inst_data)
  }

  // ================
  //   Finalizacion
  // ================

  pub(crate) fn finish(self) -> IrModule {
    self.program
  }

  // ===========
  //   Helpers
  // ===========

  fn add_inst_to_program(&mut self, data: InstData) -> InstId {
    let id = self.inst_id_generator.next_id();
    self.program.add_inst(data);
    self.append_inst_to_current_block(id);
    id
  }

  fn append_inst_to_current_block(&mut self, inst_id: InstId) {
    let is_terminator = self.program.inst(inst_id).kind.is_terminator();
    let block = self.program.block_mut(self.current_block());

    if is_terminator {
      debug_assert!(!block.has_terminator(), "el bloque ya tiene terminador");
      block.set_terminator(inst_id);
    } else {
      debug_assert!(
        !block.has_terminator(),
        "no se puede agregar una instruccion tras el terminador"
      );
      block.add_inst(inst_id);
    }
  }

  fn current_block(&self) -> BlockId {
    self.current_block.expect("debe haber un bloque actual")
  }
}
