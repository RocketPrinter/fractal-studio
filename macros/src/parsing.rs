use syn::parse::{Parse, ParseStream};
use naga_oil::compose::ShaderDefValue;
use syn::{LitBool, Token, Visibility, Ident, braced, LitStr, parenthesized};
use crate::{ShaderDefValueType, ValueEnumDeclaration, Variant, VariantsDeclaration, WgslVariants};

impl Parse for WgslVariants {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut value_enum_decls = vec![];
        let mut variants_decls = vec![];

        while !input.is_empty() {
            let fork = input.fork();
            fork.parse::<Visibility>()?;

            match fork.parse::<Ident>()?.to_string().as_str() {
                "value_enum" => value_enum_decls.push(
                    parse_value_enum_decl(input)?
                ),
                "variants" => variants_decls.push(
                    parse_variants_decl(input)?
                ),
                _ => panic!("Expected either \"value_enum\" or \"variants\"")
            }
        }

        Ok(WgslVariants { value_enum_decls, variants_decls })
    }
}

fn parse_value_enum_decl(input: ParseStream) -> syn::Result<ValueEnumDeclaration> {
    let vis = input.parse::<Visibility>()?;
    parse_str_ident(input, "value_enum")?;

    let name = input.parse::<Ident>()?;
    let codegen_name =
        if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            Some(input.parse::<Ident>()?)
        } else {None};

    input.parse::<Token![:]>()?;
    let v_type = parse_shader_def_value_type(input)?;

    let inner_input;
    braced!(inner_input in input);
    let values = parse_multiple_comma_delimited(&inner_input, |input| {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = parse_shader_def_value(input, v_type)?;
        Ok((name, value))
    })?;

    assert!(!values.is_empty(), "value_enum declaration must have at least one value!");

    Ok(ValueEnumDeclaration {
        vis,
        name,
        codegen_name,
        v_type,
        values,
    })
}

fn parse_variants_decl(input: ParseStream) -> syn::Result<VariantsDeclaration> {
    let vis = input.parse::<Visibility>()?;
    parse_str_ident(input, "variants")?;
    let name = input.parse::<Ident>()?;
    parse_str_ident(input, "from")?;
    let path_end = input.parse::<LitStr>()?.value();
    let mut path = std::env::current_dir().unwrap();
    path.push(path_end);

    let inner_input;
    braced!(inner_input in input);
    let mut variants = parse_multiple_comma_delimited(&inner_input, parse_variant )?;

    let shared = variants.iter().position(|variant|variant.get_name() == "shared")
        .map(|i| match variants.remove(i) {
            Variant::HardCoded {kvp, ..} => kvp,
            Variant::CrossProduct {..} => panic!("shared block must have hardcoded values"),
        });

    Ok(VariantsDeclaration{
        vis,
        name,
        path,
        shared,
        variants,
    })
}

fn parse_variant(input: ParseStream) -> syn::Result<Variant> {
    let name = input.parse::<Ident>()?;

    let lookahead = input.lookahead1();
    let inner_input;
    if lookahead.peek(syn::token::Brace) {

        braced!(inner_input in input);
        let kvp = parse_multiple_comma_delimited(
            &inner_input,
            |input| {
                let name = input.parse::<Ident>()?;
                input.parse::<Token![:]>()?;
                let kind = parse_shader_def_value_type(input)?;
                input.parse::<Token![=]>()?;
                let value = parse_shader_def_value(input, kind)?;

                Ok((name, value))
            }
        )?;

        Ok(Variant::HardCoded {
            name,
            kvp,
        })

    } else if lookahead.peek(syn::token::Paren) {

        parenthesized!(inner_input in input);
        let value_enums = parse_multiple_comma_delimited(
            &inner_input,
            |input| input.parse::<Ident>()
        )?;

        Ok(Variant::CrossProduct {
            name,
            value_enums,
        })
    } else {
        return Err(lookahead.error())?;
    }
}

fn parse_shader_def_value_type(input: ParseStream) -> syn::Result<ShaderDefValueType> {
    Ok(
        match input.parse::<Ident>()?.to_string().as_str() {
            "bool" => ShaderDefValueType::Bool,
            "i32" => ShaderDefValueType::I32,
            "u32" => ShaderDefValueType::U32,
            _ => Err(syn::Error::new(input.span(), "Expected bool, i32 or u32 as type of value_enum"))?,
        }
    )
}

fn parse_shader_def_value(input: ParseStream, kind: ShaderDefValueType) -> syn::Result<ShaderDefValue> {
    Ok(
        match kind {
            ShaderDefValueType::Bool =>
                ShaderDefValue::Bool(input.parse::<LitBool>()?.value),
            ShaderDefValueType::I32 =>
                ShaderDefValue::Int(input.parse::<syn::LitInt>()?.base10_parse::<i32>()?),
            ShaderDefValueType::U32 =>
                ShaderDefValue::UInt(input.parse::<syn::LitInt>()?.base10_parse::<u32>()?),
        }
    )
}

fn parse_multiple_comma_delimited<T>(input: ParseStream, item: impl Fn(ParseStream) -> syn::Result<T>) -> syn::Result<Vec<T>> {
    parse_multiple_delimited::<_, Token![,]>(input, item)
}

fn parse_multiple_delimited<T, D: Parse>(input: ParseStream, item: impl Fn(ParseStream) -> syn::Result<T>) -> syn::Result<Vec<T>> {
    let mut vec = vec![];
    while !input.is_empty() {
        vec.push(item(input)?);

        if !input.is_empty() {
            input.parse::<D>()?;
        }
    }
    Ok(vec)
}

fn parse_str_ident(input: ParseStream, ident: &str) -> syn::Result<()> {
    match input.parse::<Ident>()? == ident {
        true => Ok(()),
        false => Err(syn::Error::new(input.span(), format!("\"{}\" identifier expected", ident)))
    }
}