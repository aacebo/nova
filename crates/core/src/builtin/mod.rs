mod compare;
mod if_else;
mod not;

pub use compare::*;
pub use if_else::*;
pub use not::*;

use crate::{Action, Args, Builder, Predicate};

pub fn register(builder: Builder) -> Builder {
    builder
        .predicate("compare", |args: &Args| {
            let left = args.get_required::<String>("left")?;
            let right = args.get_required::<String>("right")?;
            let style = match args.get_or_default::<String>("op").as_str() {
                "gt" => Cmp::Gt,
                "lt" => Cmp::Lt,
                _ => Cmp::Eq,
            };

            Compare::new(left, right, style).invoke(args)
        })
        .action("if", |args: &Args| {
            let cond = args.get_required::<String>("cond")?;
            let then = args.get_required::<String>("then")?;
            let mut node = If::new(cond, then);

            if let Some(otherwise) = args.get("else").and_then(|v| v.as_str()) {
                node = node.or_else(otherwise);
            }

            node.invoke(args)
        })
        .predicate("not", |args: &Args| {
            let inner = args.get_required::<String>("of")?;
            Not::new(inner).invoke(args)
        })
}
