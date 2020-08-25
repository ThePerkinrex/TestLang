use crate::span::Span;

mod types;
pub use types::*;
// mod types_old;
mod traits;
pub use traits::*;
mod expr;
pub use expr::*;
mod item;
pub use item::*;
mod value;
pub use value::*;

pub mod intrinsics;

pub type Block = Vec<Span<Expr>>;
pub type Ident = String;
