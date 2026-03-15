// La IR de lolo-lang:
//  - es tipada,
//  - es SSA
//  - es basada en bloques

mod block;
mod builder;
mod cfg;
mod ids;
mod inst;
mod ir_invariant_error;
mod ir_source_map;
mod lowering;
mod lowering_error;
mod module;
mod pretty;
mod ssa_env;
#[cfg(test)]
mod test_helpers;
mod types;
mod value;
mod verify;

pub(crate) use lowering::LoweringCtx;
pub(crate) use module::IrModule;
