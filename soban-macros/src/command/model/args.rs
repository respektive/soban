use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    token::{Mut, Underscore},
    Error, Ident, Result, Token, Type,
};

#[derive(Clone)]
pub struct Args {
    pub ctx: Arg,
    pub orig: Arg,
    pub args: Arg,
}

impl Args {
    pub fn validate(&mut self) -> Result<()> {
        if &*self.ctx.ty != &parse_quote!(Arc<Context>) {
            let content = "first argument must have type `Arc<Context>`";

            return Err(Error::new_spanned(&self.ctx, content));
        }

        if &*self.orig.ty != &parse_quote!(CommandOrigin<'_>) {
            let content = "second argument must have type `CommandOrigin<'_>`";

            return Err(Error::new_spanned(&self.orig, content));
        }

        self.orig.ty = parse_quote!(CommandOrigin<'fut>);

        if &*self.args.ty != &parse_quote!(Args<'_>) {
            let content = "third argument must have type `Args<'_>`";

            return Err(Error::new_spanned(&self.args, content));
        }

        self.args.ty = parse_quote!(Args<'fut>);

        Ok(())
    }

    pub fn ensure_names(&mut self) {
        let ctx_name = format_ident!("ctx", span = self.ctx.name.span());
        self.ctx.name = ArgName::Ident(ctx_name);

        let orig_name = format_ident!("orig", span = self.orig.name.span());
        self.orig.name = ArgName::Ident(orig_name);

        let args_name = format_ident!("args", span = self.args.name.span());
        self.args.name = ArgName::Ident(args_name);
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let ctx = content.parse::<Arg>()?;
        let _ = content.parse::<Token![,]>()?;
        let orig = content.parse::<Arg>()?;
        let _ = content.parse::<Token![,]>()?;
        let args = content.parse::<Arg>()?;

        Ok(Self { ctx, orig, args })
    }
}

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ctx.to_tokens(tokens);
        Token![,](Span::call_site()).to_tokens(tokens);
        self.orig.to_tokens(tokens);
        Token![,](Span::call_site()).to_tokens(tokens);
        self.args.to_tokens(tokens);
    }
}

#[derive(Clone)]
pub struct Arg {
    pub mutability: Option<Mut>,
    pub name: ArgName,
    pub colon: Token![:],
    pub ty: Box<Type>,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        let mutability = input.peek(Token![mut]).then(|| input.parse()).transpose()?;
        let name = input.parse()?;
        let colon = input.parse::<Token![:]>()?;
        let ty = input.parse()?;

        Ok(Self {
            mutability,
            name,
            colon,
            ty,
        })
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            mutability,
            name,
            colon,
            ty,
        } = self;

        mutability.to_tokens(tokens);
        name.to_tokens(tokens);
        colon.to_tokens(tokens);
        ty.to_tokens(tokens);
    }
}

#[derive(Clone)]
pub enum ArgName {
    Ident(Ident),
    Wildcard(Underscore),
}

impl Parse for ArgName {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![_]) {
            input.parse().map(Self::Wildcard)
        } else {
            input.parse().map(Self::Ident)
        }
    }
}

impl ToTokens for ArgName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => tokens.extend(quote!(#ident)),
            Self::Wildcard(underscore) => tokens.extend(quote!(#underscore)),
        }
    }
}
