use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

#[proc_macro_derive(Partial)]
pub fn partial_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let partial_name = syn::Ident::new(&format!("Partial{}", name), name.span());
    let fields_enum_name = syn::Ident::new(&format!("{}Field", name), name.span());
    let missing_fields_error_name =
        syn::Ident::new(&format!("Missing{}FieldsErr", name), name.span());

    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => panic!("Partial derive only supports structs with named fields"),
        },
        _ => panic!("Partial derive only supports structs"),
    };

    // Collect field information
    let field_idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let field_idents_pascal: Vec<_> = fields
        .iter()
        .map(|f| {
            let original_name = f.ident.as_ref().unwrap().to_string();
            let pascal_case_name = to_pascal_case(&original_name);
            Ident::new(&pascal_case_name, Span::call_site())
        })
        .collect();
    let field_idents_str: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();

    let expanded = quote! {
        #[derive(Debug, Clone, Default, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #partial_name {
            #(pub #field_idents: Option<#field_types>,)*
        }

        #[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        pub enum #fields_enum_name {
            #(#field_idents_pascal),*
        }

        impl std::str::FromStr for #fields_enum_name {
            type Err = String;

            fn from_str(s: &str) -> Result< Self, Self::Err > {
                match s {
                    #(
                        #field_idents_str => Ok( Self::#field_idents_pascal ),
                    )*
                    _ => Err(format!("Unknown field: {}", s)),
                }
            }
        }
        impl std::error::Error for #fields_enum_name {}

        #[derive(Debug, Clone, Default, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #missing_fields_error_name{pub missing_fields: Vec<#fields_enum_name>,}
        impl std::error::Error for #missing_fields_error_name {}
        impl std::fmt::Display for #missing_fields_error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let missing_fields_str: Vec<String> = self.missing_fields
                    .iter()
                    .map(|field| field.to_string())
                    .collect();
                write!(f, "Missing fields: {}", missing_fields_str.join(", "))
            }
        }

        impl std::fmt::Display for #fields_enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(
                        Self::#field_idents_pascal => write!(f, #field_idents_str),
                    )*
                }
            }
        }

        impl #partial_name {
            pub fn apply_partial(self, mut against: #name) -> #name {
                #(
                if let Some(ref value) = self.#field_idents {
                    against.#field_idents = value.to_owned();
                }
                )*
                against
            }
            pub fn check_complete(&self) -> Result<(), #missing_fields_error_name> {
                let mut missing_fields = Vec::new();
                #(
                if let None = self.#field_idents {
                    missing_fields.push(#fields_enum_name::#field_idents_pascal);
                }
                )*
                match missing_fields.is_empty() {
                    true => Ok(()),
                    false => Err(#missing_fields_error_name{ missing_fields })
                }
            }

            /// Overwrites any of `self`'s fields with `other`'s non-None values
            pub fn merge(mut self, other: #partial_name, prefer_other_if_clash: bool) -> Self {
                #(
                match (other.#field_idents, &self.#field_idents, prefer_other_if_clash ) {
                    ( Some(incoming), Some(_), true ) | ( Some(incoming), None, _ ) => {self.#field_idents = Some(incoming);},
                    _ => {},
                }
                )*
                self
            }
        }

        impl From<#name> for #partial_name {
            fn from(original: #name) -> Self {
                Self {
                    #(#field_idents: Some(original.#field_idents),)*
                }
            }
        }

        impl TryFrom<#partial_name> for #name {
            type Error = #missing_fields_error_name;

            fn try_from(partial: #partial_name) -> Result<Self, Self::Error> {
                let mut missing_fields = Vec::new();

                #(
                    if partial.#field_idents.is_none() {
                        missing_fields.push(#fields_enum_name::#field_idents_pascal);
                    }
                )*

                match missing_fields.is_empty() {
                    true => Ok(Self {
                        #(#field_idents: partial.#field_idents.unwrap(),)*
                    }),
                    false => Err(#missing_fields_error_name{missing_fields}),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
