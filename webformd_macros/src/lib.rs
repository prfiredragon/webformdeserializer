extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident}; 
use syn::{parse_macro_input, Data, DeriveInput, Fields};
use syn::{Meta, NestedMeta};


#[proc_macro_derive(WebformDeserialize, attributes(webformd))]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident; // <-- This is the struct's name!
    
    // Check if the input is a named struct, otherwise panic
    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            &fields.named
        } else {
            panic!("`#[derive(OursistemDeserialize)]` only supports structs with named fields.");
        }
    } else {
        panic!("`#[derive(OursistemDeserialize)]` only supports structs.");
    };

    let mut declarations = vec![];
    let mut matches = vec![];
    let mut assignments = vec![];

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    //is_vec_result

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_name_str = field_name.to_string();

        let temp_var = format_ident!("___{}", field_name_str);

        // Parse the attribute once
        let from_str_attr = field.attrs.iter().any(|attr| {
            if attr.path.is_ident("webformd") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    return meta_list.nested.iter().any(|nested| {
                        if let NestedMeta::Meta(Meta::Path(path)) = nested {
                            return path.is_ident("from_str");
                        }
                        false
                    });
                }
            }
            false
        });

        let (is_option, inner_type_of_option) = is_option(field_ty);
        let (is_vec_result, final_ty) = is_vec(field_ty);

        if is_option {
            let (is_inner_vec, inner_final_ty) = is_vec(inner_type_of_option.unwrap());
            if is_inner_vec {
                if from_str_attr {
                    // Case: Option<Vec<T>> with from_str
                    let inner_ty_to_parse = inner_final_ty.unwrap();
                    declarations.push(quote! { let mut #temp_var: Vec<String> = Vec::new(); });
                    matches.push(quote! {
                        #field_name_str => { #temp_var.push(value.clone()); }
                    });
                    assignments.push(quote! {
                        #field_name: {
                            let parsed: Result<Vec<#inner_ty_to_parse>, _> = #temp_var
                                .into_iter()
                                .map(|s| s.parse())
                                .collect();
                            let final_vec = match parsed {
                                Ok(v) => v,
                                Err(e) => return Err(e.to_string()),
                            };
                            if final_vec.is_empty() { None } else { Some(final_vec) }
                        },
                    });
                } else {
                    // Case: Option<Vec<String>>
                    declarations.push(quote! { let mut #temp_var: Vec<String> = Vec::new(); });
                    matches.push(quote! {
                        #field_name_str => { #temp_var.push(value.clone()); }
                    });
                    assignments.push(quote! {
                        #field_name: if #temp_var.is_empty() { None } else { Some(#temp_var) },
                    });
                }
            } else {
                // Case: Option<T> (without Vec)
                declarations.push(quote! { let mut #field_name: Option<Option<String>> = None; });
                matches.push(quote! { #field_name_str => { #field_name = Some(Some(value.clone())); } });
                assignments.push(quote! { #field_name: #field_name.flatten(), });
            }
        } else if is_vec_result {
            if from_str_attr {
                // This part is for `Vec<T>` with `from_str`. The assignment is correct here.
                let final_ty = final_ty.unwrap();
                declarations.push(quote! { let mut #temp_var: Vec<String> = Vec::new(); });
                matches.push(quote! {
                    #field_name_str => { #temp_var.push(value.clone()); }
                });
                assignments.push(quote! {
                    #field_name: {
                        let parsed: Result<Vec<#final_ty>, _> = #temp_var.into_iter().map(|s| s.parse()).collect();
                        match parsed {
                            Ok(v) => v,
                            Err(e) => return Err(e.to_string()),
                        }
                    },
                });
            } else {
                // This is the section you need to fix.
                // The assignment should be direct, without `ok_or_else`.
                declarations.push(quote! { let mut #temp_var: Vec<String> = Vec::new(); });
                matches.push(quote! {
                    #field_name_str => { #temp_var.push(value.clone()); }
                });
                // Corrected assignment:
                assignments.push(quote! { #field_name: #temp_var, });
            }
        } else {
            // This is the part for a required `String`, where `ok_or_else` is correct.
            if from_str_attr {
                // ...
            } else {
                declarations.push(quote::quote! { let mut #field_name: Option<String> = None; });
                matches.push(quote::quote! {
                    #field_name_str => { #field_name = Some(value.clone()); }
                });
                assignments.push(quote::quote! { #field_name: #field_name.ok_or_else(|| format!("Missing required field: '{}'", #field_name_str))?, });
            }
        }
    }

    // This is the crucial part: using `#struct_name` to make the implementation generic.
    let expanded = quote! {
        impl #impl_generics webformd::WebFomData for #struct_name #ty_generics #where_clause {
            fn deserialize(data: &Vec<(String, String)>) -> Result<Self, String> {
                #(#declarations)*
                for (key, value) in data {
                    match key.as_str() {
                        #(#matches)*
                        _ => {}
                    }
                }
                let s = #struct_name {
                    #(#assignments)*
                };
                Ok(s)
            }
        }
    };

    expanded.into()
}


fn is_option(ty: &syn::Type) -> (bool, Option<&syn::Type>) {
    if let syn::Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            let segment = &type_path.path.segments[0];
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return (true, Some(inner_ty));
                    }
                }
            }
        }
    }
    (false, None)
}

fn is_vec(ty: &syn::Type) -> (bool, Option<&syn::Type>) {
    if let syn::Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            let segment = &type_path.path.segments[0];
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return (true, Some(inner_ty));
                    }
                }
            }
        }
    }
    (false, None)
}


