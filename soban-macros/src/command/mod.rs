pub mod attrs;
pub mod model;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Result;

use self::{attrs::CommandAttrs, model::command::CommandFn};

pub fn impl_command(cmd_attrs: CommandAttrs, cmd_fn: CommandFn) -> Result<TokenStream> {
    let CommandAttrs { aliases } = cmd_attrs;

    let CommandFn {
        vis,
        async_token,
        fn_token,
        name: cmd_ident,
        args: cmd_args,
        ret,
        body,
    } = cmd_fn;

    let mut run_args = cmd_args.clone();
    run_args.ensure_names();
    let ctx_name = &run_args.ctx.name;
    let orig_name = &run_args.orig.name;
    let args_name = &run_args.args.name;

    let ret_ty = &ret.ty;

    let cmd_name = cmd_ident.to_string();

    let static_name = format_ident!("{}", cmd_name.to_uppercase(), span = cmd_ident.span());

    let run_fn_name = format_ident!("run_{cmd_name}");

    let cmd_slice_path = quote!(crate::COMMANDS_SLICE);
    let cmd_path = quote!(crate::Command);
    let box_fut_path = quote!(::futures::future::BoxFuture);

    let tokens = quote! {
        #[linkme::distributed_slice( #cmd_slice_path )]
        pub static #static_name: #cmd_path = #cmd_path {
            name: #cmd_name,
            aliases: &[ #aliases ],
            run: #run_fn_name,
        };

        fn #run_fn_name<'fut>( #run_args ) -> #box_fut_path<'fut, #ret_ty> {
            Box::pin( #cmd_ident( #ctx_name, #orig_name, #args_name ) )
        }

        #vis #async_token #fn_token #cmd_ident <'fut> ( #cmd_args ) #ret #body
    };

    Ok(tokens)
}
