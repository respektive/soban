use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Error, LitStr, MetaList, Result, Token,
};

pub struct CommandAttrs {
    pub aliases: Punctuated<LitStr, Token![,]>,
}

impl Parse for CommandAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                aliases: Punctuated::new(),
            });
        }

        let meta = input.parse::<MetaList>()?;

        let aliases = if meta.path.is_ident("aliases") {
            meta.parse_args_with(Punctuated::parse_separated_nonempty)?
        } else {
            return Err(Error::new_spanned(meta.path, "expected `aliases`"));
        };

        Ok(Self { aliases })
    }
}
