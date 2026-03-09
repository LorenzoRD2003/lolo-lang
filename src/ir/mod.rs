// La IR de lolo-lang:
//  - es tipada,
//  - es de tres direcciones
//  - no es SSA
//  - es basada en bloques

mod block;
mod builder;
mod error;
mod ids;
mod inst;
mod ir_source_map;
mod local;
mod lowering;
mod pretty;
mod program;
mod types;
mod value;
mod verify;
mod visitor;
