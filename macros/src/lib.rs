#![allow(clippy::redundant_closure)]

use proc_macro::{TokenStream};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use naga_oil::compose::preprocess::Preprocessor;
use naga_oil::compose::ShaderDefValue;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, Ident, LitBool, LitStr, parse_macro_input, Token, Visibility};
use syn::spanned::Spanned;

#[derive(Debug)]
struct IncludeWgslVariants {
    vis: Visibility,
    name: Ident,
    path: PathBuf,
    shared_defs: Option<HashMap<String, ShaderDefValue>>,
    variants: Vec<Variant>,
}

#[derive(Debug)]
struct Variant {
    name: String,
    defs: HashMap<String, ShaderDefValue>,
}

//region ###PARSING###
impl Parse for IncludeWgslVariants {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse::<Visibility>()?;
        parse_str_ident(input, "variants")?;
        let name = input.parse::<Ident>()?;
        parse_str_ident(input, "from")?;
        let path_end = input.parse::<LitStr>()?.value();
        let mut path = std::env::current_dir().unwrap();
        path.push(path_end);

        let inner_input;
        braced!(inner_input in input);

        let mut shared_defs = None;
        let mut variants = vec![];
        while !inner_input.is_empty() {
            let variant = parse_variant(&inner_input)?;
            if variant.name == "shared" {
                shared_defs = Some(variant.defs);
            }
            else {
                variants.push(variant);
            }
            if inner_input.is_empty() {break;}
            inner_input.parse::<Token![,]>()?;
        }

        Ok(IncludeWgslVariants {
            vis,
            name,
            path,
            shared_defs,
            variants,
        })
    }
}

fn parse_str_ident(input: ParseStream, ident: &str) -> syn::Result<()> {
    match input.parse::<Ident>()? == ident {
        true => Ok(()),
        false => Err(syn::Error::new(input.span(), format!("\"{}\" identifier expected", ident)))
    }
}

fn parse_shader_def(input: ParseStream) -> syn::Result<(String,ShaderDefValue)> {
    let name = input.parse::<Ident>()?.to_string();
    input.parse::<Token![:]>()?;
    let value = match input.parse::<Ident>()?.to_string().as_str() {
        "bool" => {
            input.parse::<Token![=]>()?;
            let value = input.parse::<LitBool>()?;
            ShaderDefValue::Bool(value.value)
        },
        "i32" => {
            input.parse::<Token![=]>()?;
            // suffix will be ignored
            let value = input.parse::<syn::LitInt>()?.base10_parse::<i32>()?;
            ShaderDefValue::Int(value)
        },
        "u32" => {
            input.parse::<Token![=]>()?;
            // suffix will be ignored
            let value = input.parse::<syn::LitInt>()?.base10_parse::<u32>()?;
            ShaderDefValue::UInt(value)
        },
        _ => Err(syn::Error::new(input.span(), "Expected bool, i32 or u32 as type of shader def"))?,
    };
    Ok((name, value))
}

fn parse_variant(input: ParseStream) -> syn::Result<Variant> {
    let name = input.parse::<Ident>()?.to_string();
    input.parse::<Token![:]>()?;

    let defs_input;
    braced!(defs_input in input);

    let mut defs = HashMap::new();
    while !defs_input.is_empty() {
        let def = parse_shader_def(&defs_input)?;
        defs.insert(def.0, def.1);
        if defs_input.is_empty() { break; }
        defs_input.parse::<Token![,]>()?;
    }

    Ok(Variant{
        name, defs
    })
}
//endregion

/// ```
///include_wgsl_variants!{
///     pub variants Shader from "src/wgsl/file.wgsl" {
///         shared: {
///             Foo: bool = true,
///         },
///         Variant1 {
///             Bar: u32 = 69,
///         },
///         Variant2 {
///             Foo: bool = false,
///             Baz: i32 = true,
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn include_wgsl_variants(input: TokenStream) -> TokenStream {
    let IncludeWgslVariants { vis, name, path, shared_defs, variants } =
        parse_macro_input!(input as IncludeWgslVariants);

    println!("Processing macro {path:?}");
    let file = read_to_string(path).unwrap();
    let preproc = Preprocessor::default();

    let mut variant_names = vec![];
    let mut variant_shaders = vec![];
    for Variant{ name, mut defs } in variants {
        // we just need to merge the constant defs with the variant's defs
        for (name, value) in shared_defs.iter().flatten() {
            if !defs.contains_key(name) {
                defs.insert(name.clone(), *value);
            }
        }

        let output = preproc.preprocess(&file, &defs, false).unwrap().preprocessed_source;

        variant_names.push(Ident::new(&name, name.span()));
        variant_shaders.push(output);
    };

    let output = quote! {
        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
        #vis enum #name {
            #(#variant_names,)*
        }

        impl #name {
            #vis fn get_shader(self) -> &'static str {
                match self {
                    #(Self::#variant_names => #variant_shaders,)*
                }
            }
        }
    };

    TokenStream::from(output)
}