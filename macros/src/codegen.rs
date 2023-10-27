use std::fs::read_to_string;
use std::collections::{HashMap};
use naga_oil::compose::preprocess::Preprocessor;
use naga_oil::compose::ShaderDefValue;
use proc_macro2::{TokenStream, Ident};
use quote::{format_ident, quote, TokenStreamExt};
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
    let values = values.iter().map(|(_, value)|{
        shader_def_val_to_tokens(value)
    });

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
            Variant::HardCoded { name, kvp } => {
                defs.extend(kvp.into_iter().map(|(a, b)| (a.to_string(), b)));

                variant_decls.extend( quote! {#name,});

                let source = preproc.preprocess(&file, &defs, false).unwrap().preprocessed_source;
                // todo: labels
                match_arms.extend(quote! {Self::#name => ShaderModuleDescriptor {label:None, source: wgpu::ShaderSource::Wgsl(#source.into())},});
            }

            Variant::CrossProduct { name, value_enums } => {
                let value_enums = value_enums.iter().map(|name| {
                    let Some(v) = value_enum_decls.iter().find(|v| v.name == *name) else {
                        panic!("enum_value {name} is not defined.");
                    };
                    v
                }).collect::<Vec<_>>();

                let value_enum_types = value_enums.iter().map(|value_enum| &value_enum.name);
                variant_decls.extend( quote! {#name(#(#value_enum_types,)*,),});

                let mut match_arms = TokenStream::new();
                generate_combinations(&value_enums, &mut vec![], &preproc, &file, &name, &mut defs, &mut match_arms);

                // recursively generates all the possible combinations and writes them to the TokenStream
                fn generate_combinations(value_enums: &[&ValueEnumDeclaration], value_enum_indexes: &mut Vec<usize>,
                                         preproc: &Preprocessor, file: &str, variant_name: &Ident, defs: &mut HashMap<String, ShaderDefValue>,
                                         output_stream: &mut TokenStream) {
                    if value_enum_indexes.len() == value_enums.len() {
                        let source = preproc.preprocess(file, defs, false).unwrap().preprocessed_source;
                        let values = value_enum_indexes.iter().zip(value_enums.iter()).map(|(i, value_enum)| {
                            shader_def_val_to_tokens(&value_enum.values[*i].1)
                        });

                        // Variant (values...) => output,
                        // todo: labels
                        output_stream.extend(quote! { #variant_name( #(#values)* ) => ShaderModuleDescriptor {label:None, source: wgpu::ShaderSource::Wgsl(#source.into())}, });

                        return;
                    }

                    let i = value_enum_indexes.len();
                    let value_enum = value_enums[i];
                    defs.insert(value_enum.name.to_string(), ShaderDefValue::Int(0));

                    value_enum_indexes.push(0);
                    for j in 0..value_enum.values.len() {
                        let _ = defs.get_mut(&value_enum.name.to_string()).insert(&mut value_enum.values[j].1.clone());
                        value_enum_indexes[i] = j;

                        generate_combinations(value_enums, value_enum_indexes,
                                              preproc, file, variant_name, defs,
                                              output_stream);
                    }
                    value_enum_indexes.pop();
                }
            },
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