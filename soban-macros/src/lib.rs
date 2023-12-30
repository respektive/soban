mod command;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::command::{attrs::CommandAttrs, model::command::CommandFn};

#[proc_macro_attribute]
pub fn command(attr: TokenStream, input: TokenStream) -> TokenStream {
    let cmd_attrs = parse_macro_input!(attr as CommandAttrs);
    let cmd_fn = parse_macro_input!(input as CommandFn);

    match command::impl_command(cmd_attrs, cmd_fn) {
        Ok(result) => result.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
