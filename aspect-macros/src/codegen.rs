//! Code generation utilities for aspect weaving.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ItemFn, ReturnType};

use crate::parsing::AspectInfo;

/// Generates the aspect-woven code for a function.
pub fn generate_aspect_wrapper(aspect_info: &AspectInfo, func: &ItemFn) -> TokenStream {
    let original_fn = func;
    let fn_name = &func.sig.ident;
    let fn_vis = &func.vis;
    let fn_inputs = &func.sig.inputs;
    let fn_output = &func.sig.output;
    let fn_generics = &func.sig.generics;
    let fn_where_clause = &func.sig.generics.where_clause;
    let fn_asyncness = &func.sig.asyncness;

    let aspect_expr = &aspect_info.aspect_expr;

    // Generate the original function with a mangled name
    let original_fn_name = syn::Ident::new(
        &format!("__aspect_original_{}", fn_name),
        fn_name.span(),
    );

    let mut original_fn_renamed = original_fn.clone();
    original_fn_renamed.sig.ident = original_fn_name.clone();
    // Make the original function private
    original_fn_renamed.vis = syn::Visibility::Inherited;

    // Extract parameter names for calling the original function
    let param_names: Vec<_> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                Some(&pat_type.pat)
            } else {
                None
            }
        })
        .collect();

    // Determine the return type and if it's a Result
    let (return_type, is_result) = match fn_output {
        ReturnType::Default => (quote! { () }, false),
        ReturnType::Type(_, ty) => (quote! { #ty }, is_result_type(ty)),
    };

    // Generate aspect weaving code using around advice
    let aspect_call = if fn_asyncness.is_some() {
        // Async function handling
        generate_async_around_call(
            aspect_expr,
            &original_fn_name,
            fn_name,
            &param_names,
            &return_type,
            is_result,
        )
    } else {
        // Sync function handling
        generate_sync_around_call(
            aspect_expr,
            &original_fn_name,
            fn_name,
            &param_names,
            &return_type,
            is_result,
        )
    };

    quote! {
        // Keep the original function with mangled name
        #original_fn_renamed

        // Generate the wrapper function
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_inputs) #fn_output #fn_where_clause {
            #aspect_call
        }
    }
}

/// Generates aspect weaving code for synchronous functions using around advice.
fn generate_sync_around_call(
    aspect_expr: &Expr,
    original_fn_name: &syn::Ident,
    fn_name: &syn::Ident,
    param_names: &[&Box<syn::Pat>],
    return_type: &TokenStream,
    is_result: bool,
) -> TokenStream {
    let fn_name_str = fn_name.to_string();

    if is_result {
        // For Result types, unwrap and propagate errors properly
        quote! {
            use ::aspect_core::prelude::*;
            use ::std::any::Any;

            let __aspect = #aspect_expr;
            let __context = JoinPoint {
                function_name: #fn_name_str,
                module_path: module_path!(),
                location: Location {
                    file: file!(),
                    line: line!(),
                },
            };

            // Create ProceedingJoinPoint that wraps the original function
            let __pjp = ProceedingJoinPoint::new(
                || {
                    match #original_fn_name(#(#param_names),*) {
                        Ok(__val) => Ok(Box::new(__val) as Box<dyn Any>),
                        Err(__err) => Err(AspectError::execution(format!("{:?}", __err))),
                    }
                },
                __context,
            );

            // Call the aspect's around method
            match __aspect.around(__pjp) {
                Ok(__boxed_result) => {
                    // Downcast the result back to the original Ok type
                    let __inner = *__boxed_result
                        .downcast::<_>()
                        .expect("aspect around() returned wrong type");
                    Ok(__inner)
                }
                Err(__err) => {
                    // Convert AspectError back to the function's error type
                    Err(format!("{:?}", __err).into())
                }
            }
        }
    } else {
        // For non-Result types
        quote! {
            use ::aspect_core::prelude::*;
            use ::std::any::Any;

            let __aspect = #aspect_expr;
            let __context = JoinPoint {
                function_name: #fn_name_str,
                module_path: module_path!(),
                location: Location {
                    file: file!(),
                    line: line!(),
                },
            };

            // Create ProceedingJoinPoint that wraps the original function
            let __pjp = ProceedingJoinPoint::new(
                || {
                    let __result = #original_fn_name(#(#param_names),*);
                    Ok(Box::new(__result) as Box<dyn Any>)
                },
                __context,
            );

            // Call the aspect's around method
            match __aspect.around(__pjp) {
                Ok(__boxed_result) => {
                    // Downcast the result back to the original type
                    *__boxed_result
                        .downcast::<#return_type>()
                        .expect("aspect around() returned wrong type")
                }
                Err(__err) => {
                    panic!("aspect around() failed: {:?}", __err);
                }
            }
        }
    }
}

/// Generates aspect weaving code for asynchronous functions using around advice.
fn generate_async_around_call(
    aspect_expr: &Expr,
    original_fn_name: &syn::Ident,
    fn_name: &syn::Ident,
    param_names: &[&Box<syn::Pat>],
    _return_type: &TokenStream,
    is_result: bool,
) -> TokenStream {
    let fn_name_str = fn_name.to_string();

    // For async functions, for now we'll use a simpler approach
    // True async around advice requires async traits (not stable)
    if is_result {
        quote! {
            use ::aspect_core::prelude::*;
            use ::std::any::Any;

            let __aspect = #aspect_expr;
            let __context = JoinPoint {
                function_name: #fn_name_str,
                module_path: module_path!(),
                location: Location {
                    file: file!(),
                    line: line!(),
                },
            };

            __aspect.before(&__context);

            let __result = #original_fn_name(#(#param_names),*).await;

            match &__result {
                Ok(__val) => {
                    __aspect.after(&__context, __val as &dyn Any);
                }
                Err(__err) => {
                    let __aspect_err = AspectError::execution(format!("{:?}", __err));
                    __aspect.after_error(&__context, &__aspect_err);
                }
            }

            __result
        }
    } else {
        quote! {
            use ::aspect_core::prelude::*;
            use ::std::any::Any;

            let __aspect = #aspect_expr;
            let __context = JoinPoint {
                function_name: #fn_name_str,
                module_path: module_path!(),
                location: Location {
                    file: file!(),
                    line: line!(),
                },
            };

            __aspect.before(&__context);

            let __result = #original_fn_name(#(#param_names),*).await;

            __aspect.after(&__context, &__result as &dyn Any);

            __result
        }
    }
}

/// Checks if a type is a Result type.
fn is_result_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_result_type() {
        let result_type: syn::Type = parse_quote!(Result<i32, String>);
        assert!(is_result_type(&result_type));

        let non_result_type: syn::Type = parse_quote!(i32);
        assert!(!is_result_type(&non_result_type));
    }
}
