use core::iter::Iterator;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Component)]
/// Implements the Component trait for a custom component enum.
/// It generates the bitflags and the match code for `Component::get_type`.
/// It also implements `Component::is_marker` by defining all unit variants of the enum
/// as `marker component`.
pub fn derive_component(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let components = read_component_variants(&input);
    let fn_get_type_impl = generate_get_type_impl(name, &components);
    let fn_is_marker_impl = generate_is_marker_impl(name, &components);
    let type_ids = generate_type_ids(name, &components);
    let from_impls = generate_from_impls(name, &components);
    proc_macro::TokenStream::from(quote! {
        impl Component for #name {
            #fn_get_type_impl
            #fn_is_marker_impl
        }
        #type_ids
        #from_impls*
    })
}

fn read_component_variants(input: &DeriveInput) -> Vec<Component> {
    match &input.data {
        syn::Data::Enum(e) => {
            let mut fields = vec![];
            for v in &e.variants {
                fields.push(Component {
                    name: v.ident.clone(),
                    value_type: get_value_type(&v.fields),
                    marker: matches!(v.fields, syn::Fields::Unit),
                });
            }
            fields
        }
        _ => panic!("Component should be implemented for enums"),
    }
}

fn get_value_type(fields: &syn::Fields) -> Option<TokenStream> {
    match fields {
        syn::Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            Some(quote! {#fields})
        }
        syn::Fields::Named(_) => None, // We ignore this for now
        syn::Fields::Unit => None,
    }
}

struct Component {
    name: Ident,
    value_type: Option<TokenStream>,
    marker: bool,
}

fn generate_get_type_impl(name: &Ident, components: &[Component]) -> TokenStream {
    let cases = components.iter().map(|c| {
        let field = &c.name;
        let mut member = quote! {#name::#field};
        if !c.marker {
            member = quote! {#member(_)}
        }
        let id_name = format_ident!("{}", get_bitflag_ident(field));
        quote! {
            #member => Self::#id_name
        }
    });
    quote! {
        fn get_type(&self) -> u32 {
            match self {
                #(#cases),*
            }
        }
    }
}
fn generate_is_marker_impl(name: &Ident, components: &[Component]) -> TokenStream {
    let markers: Vec<TokenStream> = components
        .iter()
        .filter(|c| c.marker)
        .map(|c| {
            let variant = &c.name;
            quote! {#name::#variant}
        })
        .collect();
    if markers.is_empty() {
        return quote! {
            fn is_marker(&self) -> bool {
                false
            }
        };
    }
    quote! {
        fn is_marker(&self) -> bool {
            match self {
                #(#markers)|* => true,
                _ => false
            }
        }
    }
}

fn generate_type_ids(name: &Ident, components: &[Component]) -> TokenStream {
    let mut id = 1u32;
    let fields = components.iter().map(|c| {
        let field = &c.name;
        let field = get_bitflag_ident(field);
        let quote = quote! {
            pub const #field: u32 = #id;
        };
        id *= 2;
        quote
    });
    quote! {
        impl #name {
            #(#fields)*
        }
    }
}

fn get_bitflag_ident(field: &Ident) -> Ident {
    if field.to_string().len() == 1 {
        format_ident!("CONST_{}", field.to_string().to_uppercase())
    } else {
        format_ident!("{}", field.to_string().to_uppercase())
    }
}

fn generate_from_impls(name: &Ident, components: &[Component]) -> TokenStream {
    components
        .iter()
        .filter(|c| !c.marker && c.value_type.is_some())
        .filter_map(|c| {
            if let Some(value_type) = &c.value_type {
                if !is_distinct(value_type, components) {
                    return None;
                }
                let variant = &c.name;
                let value_type = &value_type;
                let quote = quote! {
                    impl From<#value_type> for #name {
                        fn from(value: #value_type) -> Self {
                            #name::#variant(value)
                        }
                    }
                };
                Some(quote)
            } else {
                None
            }
        })
        .collect()
}

fn is_distinct(value_type: &TokenStream, components: &[Component]) -> bool {
    let mut count = 0;
    for component in components {
        if let Some(t) = &component.value_type
            && t.to_string() == value_type.to_string()
        {
            count += 1;
        }
        if count > 1 {
            return false;
        }
    }
    true
}
