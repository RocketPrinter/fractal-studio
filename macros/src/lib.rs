use proc_macro::TokenStream;
use std::path::PathBuf;
use naga_oil::compose::ShaderDefValue;
use proc_macro2::Ident;
use quote::TokenStreamExt;
use syn::{parse_macro_input, Visibility};
use syn::__private::TokenStream2;
use crate::codegen::{value_enum_decl_to_tokens, variants_decl_to_tokens};

mod parsing;
mod codegen;

// todo: a lot of work went into this so it might as well be made public in a crate
/// Preprocessor macro that generates multiple variants from the same shader file using [naga_oil](https://github.com/bevyengine/naga_oil)
///```
///wgsl_variants!{
///    // used to enumerate the possible values of a shader def value
///    value_enum OWO: i32 {
///        A = 2,
///        B = -5,
///    }
///
///    // the "X as Y" allows different naming schemes on the rust and wgsl sides (CAPS vs PascalCase for example)
///    value_enum UWU as Uwu: u32 {
///        A = 69,
///        B = 420,
///    }
///
///    pub variants Shader from "src/wgsl/file.wgsl" {
///        // applies to all variants, used mainly for default values
///        shared {
///            FOO: bool = true,
///        },
///        // variant with hardcoded values
///        Variant1 {
///            BAR: u32 = 69,
///        },
///        Variant2 {
///            FOO: bool = false,
///            BAZ: i32 = true,
///        },
///        // generates 2 variants, one where OWO is 2 and one where OWO is -5
///        Variant3 (OWO)
///        // cross product of OWO and UWU, so it generates 4 variants
///        Variant4 (OWO, UWU),
///    }
///}
///```
#[proc_macro]
pub fn wgsl_variants(input: TokenStream) -> TokenStream {
    let data = parse_macro_input!(input as WgslVariants);

    let mut output = TokenStream2::new();

    output.append_all(
        data.value_enum_decls.iter().map(value_enum_decl_to_tokens)
    );

    output.append_all(
        data.variants_decls.into_iter()
            .map(|d|variants_decl_to_tokens(d, data.value_enum_decls.as_slice()))
    );

    TokenStream::from(output)
}

#[derive(Debug)]
struct WgslVariants {
    value_enum_decls: Vec<ValueEnumDeclaration>,
    variants_decls: Vec<VariantsDeclaration>,
}

#[derive(Debug)]
struct ValueEnumDeclaration {
    // it's up to the user to ensure the generated enums have the right visibility
    vis: Visibility,
    name: Ident,
    codegen_name: Option<Ident>,
    v_type: ShaderDefValueType,
    values: Vec<(Ident, ShaderDefValue)>,
}

#[derive(Debug)]
struct VariantsDeclaration {
    vis: Visibility,
    name: Ident,
    path: PathBuf,
    shared: Option<Vec<(Ident, ShaderDefValue)>>,
    variants: Vec<Variant>,
}

#[derive(Debug)]
enum Variant {
    HardCoded {
        name: Ident,
        kvp: Vec<(Ident, ShaderDefValue)>,
    },
    CrossProduct {
        name: Ident,
        value_enums: Vec<Ident>,
    },
}

impl Variant {
    pub fn get_name(&self) -> &Ident {
        let (Variant::HardCoded { name, .. } | Variant::CrossProduct { name, .. }) = self;
        name
    }
}

#[derive(Debug, Clone, Copy)]
enum ShaderDefValueType {
    Bool, I32, U32,
}