use crate::span::Span;

mod types;
pub use types::*;
mod traits;
pub use traits::*;
mod expr;
pub use expr::*;
mod item;
pub use item::*;
pub mod intrinsics;

type Block = Vec<Span<Expr>>;
pub type Ident = String;
