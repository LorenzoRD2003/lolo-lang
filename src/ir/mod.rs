// La IR de lolo-lang:
//  - es tipada,
//  - es SSA
//  - es basada en bloques

mod block;
mod builder;
mod ids;
mod inst;
#[cfg(any(test, feature = "ir-verify"))]
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
#[cfg(any(test, feature = "ir-verify"))]
mod verify;

#[cfg(test)]
pub(crate) use block::BlockData;
pub(crate) use ids::{BlockId, InstId, ValueId};
#[cfg(test)]
pub(crate) use inst::InstData;
pub(crate) use inst::{InstKind, PhiInput};
pub(crate) use lowering::LoweringCtx;
pub(crate) use module::IrModule;
#[cfg(test)]
pub(crate) use types::IrType;
#[cfg(test)]
pub(crate) use value::IrConstant;
