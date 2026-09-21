pub mod typecheck;
pub mod types;

pub use typecheck::{FuncSignature, TypeChecker, TypeError, VarInfo};
pub use types::Type;
