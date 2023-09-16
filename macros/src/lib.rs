#![allow(clippy::redundant_closure)]

use proc_macro::{TokenStream};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use naga_oil::compose::preprocess::Preprocessor;
use naga_oil::compose::ShaderDefValue as ConstantShaderDefValue;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, bracketed, Ident, LitBool, LitStr, parse_macro_input, Token, token};
use syn::spanned::Spanned;

#[derive(Debug)]
struct IncludeWgslVariants {
    /// name of generated enum
    name: Ident,
    /// path of wgsl file
    path: PathBuf,
    /// shader definition which remain constant
    const_defs: HashMap<String, ConstantShaderDefValue>,
    /// shader definition which vary per shader variant. They must be either set in the variant definition or have a default value
    variable_defs: HashMap<String, VariableShaderDefValue>,
    /// named shader variants
    variants: Vec<Variant>,
}

#[derive(Debug)]
enum ShaderDefValue {
    Constant(ConstantShaderDefValue),
    Variable(VariableShaderDefValue),
}

#[derive(Debug)]
enum VariableShaderDefValue {
    Bool{default: Option<bool>},
    Int {default: Option<i32>},
    UInt{default: Option<u32>},
}

#[derive(Debug)]
struct Variant {
    name: String,
    //
    defs: HashMap<String, ConstantShaderDefValue>,
}

//region ###PARSING###
impl Parse for IncludeWgslVariants {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_str_ident(input, "name")?;
        input.parse::<Token![:]>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        parse_str_ident(input, "path")?;
        input.parse::<Token![:]>()?;
        let path_end = input.parse::<LitStr>()?.value();
        let mut path = std::env::current_dir().unwrap();
        path.push(path_end);
        input.parse::<Token![,]>()?;

        parse_str_ident(input, "defs")?;
        input.parse::<Token![:]>()?;
        let defs_input;
        braced!(defs_input in input);
        let mut const_defs = HashMap::new();
        let mut variable_defs = HashMap::new();
        for (name, value) in defs_input.parse_terminated(parse_shader_def, Token![,])? {
            match value {
                ShaderDefValue::Constant(value) => {const_defs.insert(name, value);},
                ShaderDefValue::Variable(value) => {variable_defs.insert(name, value);},
            }

        }
        // intersect the two hashmaps to make sure there are no duplicates
        if const_defs.iter().any(|(key,_)|variable_defs.contains_key(key))
        || variable_defs.iter().any(|(key,_)|const_defs.contains_key(key)) {
            Err(syn::Error::new(input.span(), "Shader defs must be either constant or variable but not both"))?;
        }
        input.parse::<Token![,]>()?;

        parse_str_ident(input, "variants")?;
        input.parse::<Token![:]>()?;
        let variants_input;
        braced!(variants_input in input);
        // we can't use parse_terminated cause it takes a function pointer instead of closure -_-
        let mut variants = vec![];
        while !variants_input.is_empty() {
            variants.push(parse_variant(&variants_input, &variable_defs)?);
            if variants_input.is_empty() { break; }
            variants_input.parse::<Token![,]>()?;
        }

        Ok(IncludeWgslVariants {
            name,
            path,
            const_defs,
            variable_defs,
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

fn parse_shader_def(input: ParseStream) -> syn::Result<(String, ShaderDefValue)> {
    let name = input.parse::<Ident>()?.to_string();
    input.parse::<Token![:]>()?;
    let value = if !input.peek(token::Bracket) {
        // normal shader_def
        let value = match input.parse::<Ident>()?.to_string().as_str() {
            "bool" => {
                input.parse::<Token![=]>()?;
                let value = input.parse::<LitBool>()?;
                ConstantShaderDefValue::Bool(value.value)
            },
            "i32" => {
                input.parse::<Token![=]>()?;
                // suffix will be ignored
                let value = input.parse::<syn::LitInt>()?.base10_parse::<i32>()?;
                ConstantShaderDefValue::Int(value)
            },
            "u32" => {
                input.parse::<Token![=]>()?;
                // suffix will be ignored
                let value = input.parse::<syn::LitInt>()?.base10_parse::<u32>()?;
                ConstantShaderDefValue::UInt(value)
            },
            _ => Err(syn::Error::new(input.span(), "Expected bool, i32 or u32 as type of shader defs"))?,
        };
        ShaderDefValue::Constant(value)
    } else {
        // variant shader_def
        let inner;
        bracketed!(inner in input);
        let value = match inner.parse::<Ident>()?.to_string().as_str() {
            "bool" => {
                VariableShaderDefValue::Bool {
                    default: if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        Some(input.parse::<LitBool>()?.value)
                    } else { None }
                }
            },
            "i32" => {
                VariableShaderDefValue::Int {
                    default: if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        // suffix will be ignored
                        Some(input.parse::<syn::LitInt>()?.base10_parse::<i32>()?)
                    } else { None }
                }
            },
            "u32" => {
                VariableShaderDefValue::UInt {
                    default: if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        // suffix will be ignored
                        Some(input.parse::<syn::LitInt>()?.base10_parse::<u32>()?)
                    } else { None }
                }
            },
            _ => Err(syn::Error::new(input.span(), "Expected [bool], [i32] or [u32] as type of variant shader defs"))?,
        };
        ShaderDefValue::Variable(value)
    };

    Ok((name, value))
}

fn parse_variant(input: ParseStream, variable_defs: &HashMap<String, VariableShaderDefValue>) -> syn::Result<Variant> {
    let name = input.parse::<Ident>()?.to_string();
    input.parse::<Token![:]>()?;
    let defs_input;
    braced!(defs_input in input);
    let mut defs = HashMap::new();
    while !defs_input.is_empty() {
        let def = parse_variant_shader_def(&defs_input, variable_defs)?;
        defs.insert(def.0, def.1);
        if defs_input.is_empty() { break; }
        defs_input.parse::<Token![,]>()?;
    }
    // fill with defaults
    for (v_name, def) in variable_defs {
        if !defs.contains_key(v_name) {
            let Some(value) = def.get_default() else {
                return Err(syn::Error::new(input.span(), format!("Missing shader def \"{v_name}\" in variant \"{name}\"")));
            };
            defs.insert(v_name.clone(), value);
        }

    }

    Ok(Variant{
        name, defs
    })
}

/// uses variable_defs to check that the name matches and to correctly parse the value
fn parse_variant_shader_def(input: ParseStream, variable_defs: &HashMap<String, VariableShaderDefValue>) -> syn::Result<(String, ConstantShaderDefValue)> {
    let name = input.parse::<Ident>()?.to_string();
    input.parse::<Token![:]>()?;
    let variable_def = variable_defs.get(&name).ok_or_else(|| syn::Error::new(input.span(), format!("\"{}\" is not a variable shader def", name)))?;
    let value = match variable_def {
        VariableShaderDefValue::Bool { .. } => ConstantShaderDefValue::Bool(input.parse::<LitBool>()?.value),
        VariableShaderDefValue::Int { .. } => ConstantShaderDefValue::Int(input.parse::<syn::LitInt>()?.base10_parse::<i32>()?),
        VariableShaderDefValue::UInt { .. } => ConstantShaderDefValue::UInt(input.parse::<syn::LitInt>()?.base10_parse::<u32>()?),
    };
    Ok((name, value))
}

impl VariableShaderDefValue {
    fn get_default(&self) -> Option<ConstantShaderDefValue> {
        match self {
            VariableShaderDefValue::Bool { default: Some(default) } => Some(ConstantShaderDefValue::Bool(*default)),
            VariableShaderDefValue::Int { default: Some(default) } => Some(ConstantShaderDefValue::Int(*default)),
            VariableShaderDefValue::UInt { default: Some(default) } => Some(ConstantShaderDefValue::UInt(*default)),
            _ => None,
        }
    }
}
//endregion

#[proc_macro]
pub fn include_wgsl_variants(input: TokenStream) -> TokenStream {
    let data = parse_macro_input!(input as IncludeWgslVariants);
    dbg!(&data);
    let IncludeWgslVariants { name, path, const_defs, variants, .. } =
        data;

    println!("Processing macro {path:?}");
    let file = read_to_string(path).unwrap();
    let preproc = Preprocessor::default();

    let mut defs = const_defs;
    let mut variant_names = vec![];
    let mut variant_shaders = vec![];
    for Variant{ name, defs: v_defs } in variants {
        // we just need to merge the constant defs with the variant's defs
        for (name, value) in v_defs.iter() {
            defs.insert(name.clone(), *value);
        }

        let output = preproc.preprocess(&file, &defs, false).unwrap().preprocessed_source;

        variant_names.push(Ident::new(&name, name.span()));
        variant_shaders.push(output);
    };

    let output = quote! {
        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
        pub enum #name {
            #(#variant_names,)*
        }

        impl #name {
            pub fn get_shader(self) -> &'static str {
                match self {
                    #(Self::#variant_names => #variant_shaders,)*
                }
            }
        }
    };

    dbg!(&output);

    TokenStream::from(output)
}