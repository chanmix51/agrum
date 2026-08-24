//! Derive macros for agrum SQL entities.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, TypePath};

/// Derive [`agrum::SqlEntity`](https://docs.rs/agrum) and `Structured` for a struct.
///
/// Field names are used as SQL column names. SQL types are inferred from the
/// Rust field types (`Uuid` → `uuid`, `String` → `text`, …). `Option<T>` uses
/// the inferred type of `T`.
#[proc_macro_derive(SqlEntity)]
pub fn derive_sql_entity(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match expand_sql_entity(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_sql_entity(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "#[derive(SqlEntity)] can only be used on structs",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(
            input,
            "#[derive(SqlEntity)] requires a struct with named fields",
        ));
    };

    let mut structure_entries = Vec::new();
    let mut hydrate_fields = Vec::new();

    for field in &fields.named {
        let ident = field
            .ident
            .as_ref()
            .expect("named fields always have an ident");
        let column = ident.to_string();
        let sql_type = rust_type_to_sql(&field.ty)?;

        structure_entries.push(quote! { (#column, #sql_type) });
        hydrate_fields.push(quote! { #ident: row.get(#column) });
    }

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::agrum::Structured for #name #ty_generics #where_clause {
            fn get_structure() -> ::agrum::Structure {
                ::agrum::Structure::new(&[#(#structure_entries),*])
            }
        }

        #[automatically_derived]
        impl #impl_generics ::agrum::SqlEntity for #name #ty_generics #where_clause {
            fn get_projection() -> ::agrum::Projection<Self> {
                ::agrum::Projection::default()
            }

            fn hydrate(
                row: &::agrum::__private::Row,
            ) -> ::std::result::Result<Self, ::agrum::HydrationError> {
                Ok(Self {
                    #(#hydrate_fields),*
                })
            }
        }
    })
}

fn rust_type_to_sql(ty: &Type) -> syn::Result<String> {
    let Type::Path(type_path) = ty else {
        return Err(unsupported_field_type(ty));
    };

    if let Some(inner) = option_inner(type_path) {
        return rust_type_to_sql(inner);
    }

    let ident = last_path_ident(type_path).ok_or_else(|| unsupported_field_type(ty))?;
    let sql_type = match ident.as_str() {
        "Uuid" => "uuid",
        "String" => "text",
        "bool" => "bool",
        "i16" => "smallint",
        "i32" => "integer",
        "i64" => "bigint",
        "f32" => "real",
        "f64" => "double precision",
        _ => return Err(unsupported_field_type(ty)),
    };

    Ok(sql_type.to_string())
}

fn option_inner(type_path: &TypePath) -> Option<&Type> {
    let last = type_path.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &last.arguments else {
        return None;
    };

    arguments.args.iter().find_map(|argument| match argument {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}

fn last_path_ident(type_path: &TypePath) -> Option<String> {
    type_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn unsupported_field_type(ty: &Type) -> syn::Error {
    syn::Error::new_spanned(
        ty,
        "cannot infer a SQL type from this field type for #[derive(SqlEntity)]",
    )
}
