#![feature(try_blocks)]
#![feature(drain_filter)]
#![feature(box_syntax)]
#![feature(fn_traits)]
#![feature(label_break_value)]

pub(crate) mod conductors;
mod constructors;
pub(crate) mod entities;
pub(crate) mod handlers;
pub(crate) mod repositories;

// FIXME: Resultの多用が酷いのでpanic!を検討しましょう

pub use constructors::*;
