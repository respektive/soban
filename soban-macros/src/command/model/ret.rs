use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    token::RArrow,
    Error, Result, Token, Type,
};

#[derive(Clone)]
pub struct ReturnResult {
    pub arrow: RArrow,
    pub ty: Box<Type>,
}

impl ReturnResult {
    pub fn validate(&mut self) -> Result<()> {
        let expected_ty = parse_quote!(Result<()>);

        if &*self.ty != &expected_ty {
            return Err(Error::new_spanned(&self.ty, "expected `Result<()>`"));
        }

        Ok(())
    }
}

impl Parse for ReturnResult {
    fn parse(input: ParseStream) -> Result<Self> {
        let arrow = input.parse::<Token![->]>()?;
        let ty = input.parse::<Box<Type>>()?;

        Ok(Self { arrow, ty })
    }
}

impl ToTokens for ReturnResult {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.arrow.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}
