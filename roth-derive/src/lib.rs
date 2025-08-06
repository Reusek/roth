use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Fields, Lit, parse_macro_input};

/// Derive macro for automatically implementing stack_effect() method on enums.
///
/// Usage:
/// ```rust
/// #[derive(StackEffect)]
/// enum MyEnum {
///     #[stack_effect(consumes = 1, produces = 2)]
///     Dup,
///     #[stack_effect(consumes = 2, produces = 1)]
///     Add,
///     // Variants without attributes default to consumes = 0, produces = 0
///     Nop,
/// }
/// ```
#[proc_macro_derive(StackEffect, attributes(stack_effect))]
pub fn derive_stack_effect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return quote! {
            compile_error!("StackEffect can only be derived for enums");
        }
        .into();
    };

    let mut match_arms = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;

        // Parse stack effect attributes
        let mut consumes = 0;
        let mut produces = 0;

        for attr in &variant.attrs {
            if attr.path().is_ident("stack_effect") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("consumes") {
                        let value: Expr = meta.value()?.parse()?;
                        if let Expr::Lit(expr_lit) = value {
                            if let Lit::Int(lit_int) = expr_lit.lit {
                                consumes = lit_int.base10_parse::<usize>().unwrap_or(0);
                            }
                        }
                    } else if meta.path.is_ident("produces") {
                        let value: Expr = meta.value()?.parse()?;
                        if let Expr::Lit(expr_lit) = value {
                            if let Lit::Int(lit_int) = expr_lit.lit {
                                produces = lit_int.base10_parse::<usize>().unwrap_or(0);
                            }
                        }
                    }
                    Ok(())
                });
            }
        }

        // Generate match pattern based on variant fields
        let pattern = match &variant.fields {
            Fields::Unit => quote! { #name::#variant_name },
            Fields::Unnamed(_) => quote! { #name::#variant_name(..) },
            Fields::Named(_) => quote! { #name::#variant_name { .. } },
        };

        match_arms.push(quote! {
            #pattern => StackEffect {
                consumes: #consumes,
                produces: #produces,
            },
        });
    }

    let expanded = quote! {
        impl #name {
            pub fn stack_effect(&self) -> StackEffect {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

