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
    proc_macro::TokenStream::from(quote! {
        impl Component for #name {
            #fn_get_type_impl
            #fn_is_marker_impl
        }
        #type_ids
    })
}

fn read_component_variants(input: &DeriveInput) -> Vec<Component> {
    match &input.data {
        syn::Data::Enum(e) => {
            let mut fields = vec![];
            for v in &e.variants {
                fields.push(Component {
                    name: v.ident.clone(),
                    marker: matches!(v.fields, syn::Fields::Unit),
                });
            }
            fields
        }
        _ => panic!("Component should be implemented for enums"),
    }
}

struct Component {
    name: Ident,
    marker: bool,
}

fn generate_get_type_impl(name: &Ident, components: &[Component]) -> TokenStream {
    let cases = components.iter().map(|c| {
        let field = &c.name;
        let mut member = quote! {#name::#field};
        if !c.marker {
            member = quote! {#member(_)}
        }
        let id_name = format_ident!("{}", field.to_string().to_uppercase());
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
        let field = format_ident!("{}", field.to_string().to_uppercase());
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
