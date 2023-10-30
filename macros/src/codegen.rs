use std::fs::read_to_string;
use std::collections::{HashMap};
use naga_oil::compose::preprocess::Preprocessor;
use naga_oil::compose::ShaderDefValue;
use proc_macro2::{TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Lit, LitBool, LitInt};
use crate::{ShaderDefValueType, ValueEnumDeclaration, VariantsDeclaration, Variant};

pub(super) fn value_enum_decl_to_tokens(decl: &ValueEnumDeclaration) -> TokenStream {
    let ValueEnumDeclaration {
        vis, name, codegen_name, v_type, values
    } = decl;

    let name = codegen_name.as_ref().unwrap_or(name);
    let name = format_ident!("{}", name);

    let kind = match v_type {
        ShaderDefValueType::Bool => "bool",
        ShaderDefValueType::I32 => "i32",
        ShaderDefValueType::U32 => "u32",
    };
    let kind = format_ident!("{kind}");

    let names = values.iter().map(|(name,_)|name);
    let names2 = names.clone();
    let names3 = names.clone();
    let values = values.iter().map(|(_, value)|{
        shader_def_val_to_tokens(value)
    }).collect::<Vec<_>>();

    quote!{
        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #vis enum #name {
            #(#names,)*
        }

        impl #name {
            pub fn get_value(self) -> #kind {
                match self {
                    #(Self::#names2=>#values,)*
                }
            }
        }

        impl TryFrom<#kind> for #name {
            type Error = ();

            fn try_from(value: #kind) -> Result<Self, Self::Error> {
                Ok(match value {
                    #(#values=>Self::#names3,)*
                    _ => return Err(())
                })
            }
        }
    }
}

pub(super) fn variants_decl_to_tokens(decl: VariantsDeclaration, value_enum_decls: &[ValueEnumDeclaration]) -> TokenStream {
    let VariantsDeclaration{
        vis, name, path, shared, variants
    } = decl;

    println!("Processing file \"{path:?}\"");
    let file = read_to_string(&path).unwrap();
    let preproc = Preprocessor::default();
    let mut defs = HashMap::new();

    let mut variant_decls = TokenStream::new();
    let mut match_arms = TokenStream::new();

    // processes each variant, returning a pair of tokens to be injected in the enum declaration and match
    for variant in variants {
        // we reset the hashmap with the shared values
        defs.clear();
        defs.extend(shared.iter().flatten().map(|(a, b)| (a.to_string(), *b)));

        match variant {
            Variant::HardCoded { name: variant_name, kvp } => {
                defs.extend(kvp.into_iter().map(|(a, b)| (a.to_string(), b)));

                variant_decls.extend( quote! {#variant_name,});

                let source = preproc.preprocess(&file, &defs, false).unwrap().preprocessed_source;
                // todo: labels
                match_arms.extend(quote! {Self::#variant_name => ShaderModuleDescriptor {label:None, source: wgpu::ShaderSource::Wgsl(#source.into())},});
            }

            Variant::CrossProduct { name: variant_name, value_enums } => {
                let value_enums = value_enums.iter().map(|name| {
                    let Some(v) = value_enum_decls.iter().find(|v| v.get_codegen_name() == name) else {
                        panic!("enum_value {name} is not defined.");
                    };
                    v
                }).collect::<Vec<_>>();

                let value_enum_types = value_enums.iter()
                    .map(|value_enum| value_enum.get_codegen_name());
                variant_decls.extend( quote! {#variant_name(#(#value_enum_types,)*),});

                // populate with initial values
                defs.extend(value_enums.iter().map(|value_enum|
                    ( value_enum.name.to_string(), value_enum.values[0].1)
                ));

                // value_enum_indexes is like a number where each "digit" has a separate base, so to generate all the possible combinations we just repeatedly "add one"
                let mut value_enum_indexes = vec![0;value_enums.len()];
                let value_enum_keys = value_enums.iter().map(|ve|ve.name.to_string()).collect::<Vec<_>>();
                'outer: loop {

                    let source = preproc.preprocess(file.as_str(), &defs, false).unwrap().preprocessed_source;

                    let values = value_enum_indexes.iter().zip(value_enums.iter()).map(|(i, value_enum)| {
                        let enum_type = value_enum.get_codegen_name();
                        let enum_variant = &value_enum.values[*i].0;

                        quote!{#enum_type::#enum_variant}
                    });

                    // Variant (Value_enum::value, ...) => output,
                    // todo: labels
                    match_arms.extend(quote! { Self::#variant_name( #(#values,)* ) => ShaderModuleDescriptor {label:None, source: wgpu::ShaderSource::Wgsl(#source.into())}, });

                    let mut index = value_enums.len()-1;
                    loop {
                        let value_enum = value_enums[index];
                        if value_enum_indexes[index] + 1 == value_enums[index].values.len() {
                            // "carry" to next "digit" or break if index is 0 (we generated all the possible values)
                            if index == 0 {break 'outer}
                            value_enum_indexes[index] = 0;
                            *defs.get_mut(value_enum_keys[index].as_str()).unwrap() = value_enum.values[0].1;
                            index -=1;
                        } else {
                            // increase just this one and continue generating
                            value_enum_indexes[index] += 1;
                            *defs.get_mut(value_enum_keys[index].as_str()).unwrap() = value_enum.values[value_enum_indexes[index]].1;
                            break;
                        }
                    }
                }
            }
        }
    }

    let path = path.to_string_lossy();

    quote!{
        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #vis enum #name {
            #variant_decls
        }

        impl #name {
            pub fn get_shader(self) -> wgpu::ShaderModuleDescriptor<'static> {
                match self {
                    #match_arms
                }
            }

            pub fn get_raw_shader(self) -> &'static str {
                // the include_str! forces the compiler to watch the file for changes and reruns the macro if any happen so this function is actually very important
                include_str!(#path)
            }
        }
    }
}

fn shader_def_val_to_tokens(val: &ShaderDefValue) -> Lit {
    // todo: sketchy
    match val {
        ShaderDefValue::Bool(v) => Lit::Bool(LitBool::new(*v, "".span())),
        ShaderDefValue::Int(v) => {
            let s = format!("{v}i32");
            Lit::Int(LitInt::new(s.as_str(), s.span()))
        },
        ShaderDefValue::UInt(v) => {
            let s = format!("{v}u32");
            Lit::Int(LitInt::new(s.as_str(), s.span()))
        },
    }
}

/*
struct AppendHelper(Cell<Option<Box<dyn FnOnce(&mut TokenStream)>>>);

impl AppendHelper {
    pub fn new(func: Box<dyn FnOnce(&mut TokenStream)>) -> Self { AppendHelper(Cell::new(Some(func))) }
}

impl ToTokens for AppendHelper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(f) = self.0.replace(None) {
            f(tokens);
        }
    }
}
*/