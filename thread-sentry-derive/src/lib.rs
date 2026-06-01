use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_attribute]
pub fn sentry_track(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let name = &input.ident;
                let vis = &input.vis;

                let original_fields = &fields.named;

                let new_fields = original_fields.iter().map(|f| {
                    let field_vis = &f.vis;
                    let field_name = f.ident.as_ref().unwrap();
                    let field_ty = &f.ty;

                    quote! {
                        #field_vis #field_name: thread_sentry::SentryField<#field_ty>
                    }
                });

                let field_names = original_fields.iter().map(|f| f.ident.as_ref().unwrap());

                let getter_impls = original_fields.iter().map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let field_ty = &f.ty;
                    let method_name = syn::Ident::new(&format!("get_{}", field_name), field_name.span());
                    
                    quote! {
                        pub fn #method_name(&self) -> thread_sentry::SentryFieldGuard<'_, #field_ty> {
                            self.#field_name.get()
                        }
                    }
                });

                let setter_impls = original_fields.iter().map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let field_ty = &f.ty;
                    let method_name =
                        syn::Ident::new(&format!("set_{}", field_name), field_name.span());

                    quote! {
                        pub fn #method_name(&mut self, value: #field_ty) {
                            self.#field_name.set(value);
                        }
                    }
                });

                let default_init = original_fields.iter().map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    quote! {
                        #field_name: thread_sentry::SentryField::new(Default::default())
                    }
                });

                let expanded = quote! {
                    #vis struct #name {
                        #(#new_fields),*
                    }

                    impl #name {
                        pub fn new() -> Self {
                            Self {
                                #(#default_init),*
                            }
                        }

                        #(#getter_impls)*
                        #(#setter_impls)*
                    }

                    impl Default for #name {
                        fn default() -> Self {
                            Self::new()
                        }
                    }
                };

                TokenStream::from(expanded)
            }
            _ => panic!("#[sentry_track] only supports structs with named fields"),
        },
        _ => panic!("#[sentry_track] only supports structs"),
    }
}
