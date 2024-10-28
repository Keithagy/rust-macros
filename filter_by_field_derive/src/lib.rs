use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

fn to_pascal_case(s: &str) -> String {
    let numbers = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let ok: impl Iterator<Item = &i32> = numbers.iter().map(|f| f);
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

#[proc_macro_derive(FilterByField)]
pub fn filter_by_field_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let enum_name = format_ident!("{}Field", name);

    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => panic!("FieldFilter only supports structs with named fields"),
        },
        _ => panic!("FieldFilter only supports structs"),
    };

    let field_idents: Vec<_> = fields
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

    let expanded = quote! {};

    TokenStream::from(expanded)
}
