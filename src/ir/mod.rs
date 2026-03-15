// La IR de lolo-lang:
//  - es tipada,
//  - es SSA
//  - es basada en bloques

mod block;
mod builder;
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
pub(crate) mod test_helpers;
mod types;
mod value;
mod verify;

pub(crate) use lowering::LoweringCtx;
#[cfg(test)]
pub(crate) use block::BlockData;
pub(crate) use ids::BlockId;
#[cfg(test)]
pub(crate) use ids::{InstId, ValueId};
#[cfg(test)]
pub(crate) use inst::InstData;
pub(crate) use inst::InstKind;
pub(crate) use ir_invariant_error::IrInvariantError;
pub(crate) use module::IrModule;
#[cfg(test)]
pub(crate) use types::IrType;
#[cfg(test)]
pub(crate) use value::IrConstant;
