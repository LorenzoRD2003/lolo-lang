// Responsabilidad: Renderizar la IR para debugging.
// - imprimir un modulo completo
// - imprimir bloques e instrucciones
// - mostrar tipos
// - mostrar nombres amigables de locales/valores

use crate::ir::{
  IrModule,
  ids::{BlockId, InstId},
};

impl IrModule {
  pub(crate) fn pretty(&self) -> String {
    let mut output = String::new();
    self.print_module_header(&mut output);
    output.push('\n');

    for block_index in 0..self.block_count() {
      self.print_block(BlockId(block_index), &mut output);
      if block_index + 1 < self.block_count() {
        output.push('\n');
      }
    }

    output
  }

  fn print_module_header(&self, output: &mut String) {
    let module_header = format!(
      "module {} -> {} entry {}",
      self.name(),
      self.return_type(),
      self.entry_block()
    );
    output.push_str(&module_header);
  }

  fn print_block(&self, id: BlockId, output: &mut String) {
    output.push_str(&format!("{id}:\n"));
    let block = self.block(id);

    for phi in block.phis() {
      self.print_inst(*phi, output);
    }

    for inst in block.insts() {
      self.print_inst(*inst, output);
    }

    if block.has_terminator() {
      self.print_inst(block.terminator(), output);
    } else {
      output.push_str("  <missing terminator>\n");
    }
  }

  fn print_inst(&self, id: InstId, output: &mut String) {
    let inst = self.inst(id);
    output.push_str(&format!("  {inst}\n"));
  }
}

#[cfg(test)]
mod tests;
