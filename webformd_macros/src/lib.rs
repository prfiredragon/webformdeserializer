extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident}; 
use syn::{parse_macro_input, Data, DeriveInput, Fields};



#[proc_macro_derive(WebformDeserialize)]
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

    //is_vec_result

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_name_str = field_name.to_string();
        
        // Create a temporary variable to hold collected values
        let temp_var = format_ident!("___{}", field_name_str);

        let (is_option, inner_type_of_option) = is_option(field_ty);
        let (is_vec_result, _) = is_vec(field_ty);

        if is_option {
            let (is_inner_vec, _) = is_vec(inner_type_of_option.unwrap());
            if is_inner_vec {
                // 1. Handle Option<Vec<String>>
                declarations.push(quote::quote! { let mut #temp_var: Vec<String> = Vec::new(); });
                matches.push(quote::quote! {
                    #field_name_str => {
                        #temp_var.push(value.clone());
                    }
                });
                assignments.push(quote::quote! { 
                    #field_name: if #temp_var.is_empty() { None } else { Some(#temp_var) }, 
                });
            } else {
                // 3. Handle Option<String>
                declarations.push(quote::quote! { let mut #field_name: Option<Option<String>> = None; });
                matches.push(quote::quote! {
                    #field_name_str => { #field_name = Some(Some(value.clone())); }
                });
                assignments.push(quote::quote! { #field_name: #field_name.flatten(), });
            }
        } else if is_vec_result {
            // 2. Handle Vec<String>
            declarations.push(quote::quote! { let mut #temp_var: Vec<String> = Vec::new(); });
            matches.push(quote::quote! {
                #field_name_str => {
                    #temp_var.push(value.clone());
                }
            });
            assignments.push(quote::quote! { #field_name: #temp_var, });
        } else {
            // 4. Handle String (catch-all)
            declarations.push(quote::quote! { let mut #field_name: Option<String> = None; });
            matches.push(quote::quote! {
                #field_name_str => { #field_name = Some(value.clone()); }
            });
            assignments.push(quote::quote! { #field_name: #field_name.ok_or_else(|| format!("Missing required field: '{}'", #field_name_str))?, });
        }
    }

    // This is the crucial part: using `#struct_name` to make the implementation generic.
    let expanded = quote! {
        impl WebFomData for #struct_name {
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