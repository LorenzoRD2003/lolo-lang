// Responsabilidad: Dar una API ergonomica para construir IR sin andar manipulando todos los `Vec` a mano.

use crate::{
  ast::{BinaryOp, UnaryOp},
  common::{IdGenerator, IncrementalIdGenerator},
  ir::{
    block::BlockData,
    ids::{BlockId, InstId, LocalId, ValueId},
    inst::{InstData, InstKind},
    ir_source_map::IrSourceMap,
    local::LocalData,
    program::Program,
    types::IrType,
    value::{Constant, IrOperand, ValueData, ValueKind},
  },
};

#[derive(Debug, Clone)]
pub(crate) struct ProgramBuilder {
  program: Program,
  /// bloque en el cual el builder esta insertando instrucciones ahora
  current_block: Option<BlockId>,
  local_id_generator: IncrementalIdGenerator<LocalId>,
  inst_id_generator: IncrementalIdGenerator<InstId>,
  block_id_generator: IncrementalIdGenerator<BlockId>,
  value_id_generator: IncrementalIdGenerator<ValueId>,
  /// mapa entrela IR y el codigo fuente
  ir_source_map: IrSourceMap,
}

impl ProgramBuilder {
  pub(crate) fn new(name: String, return_type: IrType) -> Self {
    let program = Program::new(name, return_type);
    let mut program_builder = Self {
      current_block: None,
      program,
      local_id_generator: IncrementalIdGenerator::new(),
      inst_id_generator: IncrementalIdGenerator::new(),
      block_id_generator: IncrementalIdGenerator::new(),
      value_id_generator: IncrementalIdGenerator::new(),
      ir_source_map: IrSourceMap::new(),
    };

    let entry_block = program_builder.create_block();
    program_builder.program.set_entry_block(entry_block);
    program_builder.current_block = Some(entry_block);

    program_builder
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
  fn switch_to_block(&mut self, id: BlockId) {
    self.current_block = Some(id)
  }

  pub(crate) fn create_local(
    &mut self,
    name: impl Into<String>,
    ty: IrType,
    mutable: bool,
  ) -> LocalId {
    let local_data = LocalData::new(name.into(), ty, mutable);
    self.program.add_local(local_data);
    self.local_id_generator.next_id()
  }

  // ========================================
  //   instrucciones de creacion de valores
  // ========================================

  pub(crate) fn emit_const(&mut self, c: Constant) -> ValueId {
    let value_id = self.value_id_generator.next_id();
    let value_data = c.as_value();
    self.program.add_value(value_data);

    let inst_data = InstData::inst_with_result(value_id, InstKind::Const(c));
    self.add_inst_to_program(inst_data);

    value_id
  }

  pub(crate) fn emit_load(&mut self, local: LocalId) -> ValueId {
    let value_id = self.value_id_generator.next_id();
    let local_ty = self.program.local(local).ty().clone();
    let value_data = ValueData::new(local_ty, ValueKind::Local(local));
    self.program.add_value(value_data);
    let inst_data = InstData::inst_with_result(value_id, InstKind::Load(local));
    self.add_inst_to_program(inst_data);

    value_id
  }

  pub fn emit_store(&mut self, local: LocalId, value: IrOperand) -> InstId {
    let inst_data = InstData::inst_without_result(InstKind::Store {
      target: local,
      operand: value,
    });
    self.add_inst_to_program(inst_data)
  }

  pub fn emit_unary(&mut self, op: UnaryOp, operand: IrOperand) -> ValueId {
    let value_id = self.value_id_generator.next_id();
    let value_data = ValueData::new(op.result_type().into(), ValueKind::InstResult);
    self.program.add_value(value_data);
    let inst_data = InstData::inst_with_result(value_id, InstKind::Unary { op, operand });
    self.add_inst_to_program(inst_data);

    value_id
  }

  pub fn emit_binary(&mut self, op: BinaryOp, lhs: IrOperand, rhs: IrOperand) -> ValueId {
    let value_id = self.value_id_generator.next_id();
    let value_data = ValueData::new(op.result_type().into(), ValueKind::InstResult);
    self.program.add_value(value_data);
    let inst_data = InstData::inst_with_result(value_id, InstKind::Binary { op, lhs, rhs });
    self.add_inst_to_program(inst_data);

    value_id
  }

  // ============================
  //   Creacion de terminadores
  // ============================

  pub fn emit_jump(&mut self, target: BlockId) -> InstId {
    let inst_data = InstData::inst_without_result(InstKind::Jump { target });
    self.add_inst_to_program(inst_data)
  }

  pub fn emit_branch(
    &mut self,
    condition: IrOperand,
    if_block: BlockId,
    else_block: BlockId,
  ) -> InstId {
    let inst_data = InstData::inst_without_result(InstKind::Branch {
      condition,
      if_block,
      else_block,
    });
    self.add_inst_to_program(inst_data)
  }

  pub fn emit_return(&mut self, value: Option<IrOperand>) -> InstId {
    let inst_data = InstData::inst_without_result(InstKind::Return { value });
    self.add_inst_to_program(inst_data)
  }

  // ================
  //   Finalizacion
  // ================

  pub(crate) fn finish(self) -> Program {
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
    let block = match self.current_block {
      Some(id) => self.program.block_mut(id),
      None => unreachable!(),
    };

    if is_terminator {
      debug_assert!(!block.has_terminator(), "el bloque ya tiene terminador");
      block.set_terminator(inst_id);
    } else {
      debug_assert!(
        block.has_terminator(),
        "no se puede agregar una instruccion tras el terminador"
      );
      block.add_inst(inst_id);
    }
  }
}
