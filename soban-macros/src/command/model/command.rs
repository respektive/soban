use syn::{
    parse::{Parse, ParseStream},
    token::{Async, Fn},
    Block, Ident, Result, Token, Visibility,
};

use crate::command::model::{args::Args, ret::ReturnResult};

pub struct CommandFn {
    pub vis: Visibility,
    pub async_token: Async,
    pub fn_token: Fn,
    pub name: Ident,
    pub args: Args,
    pub ret: ReturnResult,
    pub body: Block,
}

impl Parse for CommandFn {
    fn parse(input: ParseStream) -> Result<Self> {
        // pub / nothing
        let vis = input.parse::<Visibility>()?;

        // async
        let async_token = input.parse::<Token![async]>()?;

        // fn
        let fn_token = input.parse::<Token![fn]>()?;

        // name
        let name = input.parse::<Ident>()?;

        // (Arc<Context>, CommandOrigin<'_>, &str)
        let mut args = input.parse::<Args>()?;
        args.validate()?;

        // -> Result<()>
        let mut ret = input.parse::<ReturnResult>()?;
        ret.validate()?;

        // { ... }
        let body = input.parse::<Block>()?;

        Ok(Self {
            vis,
            async_token,
            fn_token,
            name,
            args,
            ret,
            body,
        })
    }
}
