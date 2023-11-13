// Derived from the heapsize example from the syn crate:
//     https://github.com/dtolnay/syn
//     Apache-2.0, MIT dual license

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam,
    Generics, Index,
};

#[proc_macro_derive(CommandParser)]
pub fn command_parser_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let parse_body = parse_fields(&input.data);

    let expanded = quote! {
        impl #impl_generics aline::CommandParser for #name #ty_generics #where_clause {
            fn name(&self) -> &str {
                "#name"
            }

            fn parse(&mut self, args: &[&str]) -> aline::Result<()> {
                #parse_body

                Ok(())
            }
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(heapsize::HeapSize));
        }
    }
    generics
}

fn parse_fields(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let parse_fragments = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {f.span()=>
                        let (args, arg) = aline::internal::next_arg(args)?;
                        self.#name = arg.parse().map_err(|_| aline::Error::ArgumentParseError)?;
                    }
                });
                quote! {
                    #(#parse_fragments)*

                    if !args.is_empty() {
                        return Err(aline::Error::UnusedArguments);
                    }
                }
            }
            Fields::Unnamed(ref _fields) => {
                unimplemented!()
            }
            Fields::Unit => {
                unimplemented!()
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
