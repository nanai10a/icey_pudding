#![feature(try_blocks)]
#![feature(box_syntax)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(type_alias_impl_trait)]

extern crate alloc;

pub(crate) mod cmds;
pub(crate) mod conductors;
mod constructors;
pub(crate) mod controllers;
pub(crate) mod entities;
pub(crate) mod interactors;
pub(crate) mod presenters;
pub(crate) mod repositories;
pub(crate) mod usecases;
pub(crate) mod utils;

pub use constructors::*;
