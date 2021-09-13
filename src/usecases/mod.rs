macro_rules! usecase {
    ($n:ident : { $( $i:tt )* } => { $( $o:tt )* }) => {
        pub mod $n {
            use crate::entities;

            pub trait Usecase {
                fn handle(&self, data: Input) -> ::anyhow::Result<()>;
            }

            pub struct Input { $( $i )* }

            pub struct Output { $( $o )* }
        }
    };
}

pub mod content;
pub mod user;
