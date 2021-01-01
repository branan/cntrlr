// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Support macros for Cntrlr

#![deny(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Error as ParseError, Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    FnArg, Ident, ItemFn, ItemUse, Pat, ReturnType, Type,
};

struct IdentList {
    boards: Punctuated<Ident, Comma>,
}

impl Parse for IdentList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(IdentList {
            boards: input.parse_terminated(Ident::parse)?,
        })
    }
}

/// Add a function to the prelude
///
/// This macro generates the appropriate attributes for a function to
/// be added to the Cntrlr prelude.
#[proc_macro_attribute]
pub fn prelude_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_use = parse_macro_input!(input as ItemUse);
    let boards = parse_macro_input!(args as IdentList);

    let cfgs = boards
        .boards
        .iter()
        .map(|board| {
            let board_name = format!("{}", board);
            quote!(board = #board_name)
        })
        .collect::<Vec<_>>();

    quote!(
        #[cfg(any(#(#cfgs),*, doc))]
        #[cfg_attr(feature = "doc-cfg", doc(cfg(any(#(#cfgs),*))))]
        #input_use
    )
    .into()
}

/// Add a board function to a module
///
/// This macro generates an implementation of the marked function,
/// which defers to the appropriate board implementation based on
/// compile-time configuration.
#[proc_macro_attribute]
pub fn board_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;
    let boards = parse_macro_input!(args as IdentList);
    let module = &boards.boards[0];

    let cfgs = boards
        .boards
        .iter()
        .skip(1)
        .map(|board| {
            let board_name = format!("{}", board);
            quote!(board = #board_name)
        })
        .collect::<Vec<_>>();

    // This isn't super robust, but good enough for what we need to do.
    let args = sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat) => match *pat.pat {
                Pat::Ident(ref ident) => Some(ident.clone()),
                _ => None,
            },
        })
        .collect::<Vec<_>>();

    let impls = boards
        .boards
        .iter()
        .skip(1)
        .map(|board| {
            let board_name = format!("{}", board);
            quote!(
            #[cfg(board = #board_name)]
            {
                crate::hw::board::#board::#module::#fn_name(#(#args),*)
            }
            )
        })
        .collect::<Vec<_>>();

    quote!(
        #[cfg(any(#(#cfgs),*, doc))]
        #[cfg_attr(feature = "doc-cfg", doc(cfg(any(#(#cfgs),*))))]
        #(#attrs)*
        #vis #sig {
            #(#impls)*
        }
    )
    .into()
}

/// The main task of a Cntrlr application
///
/// This macro defines a startup routine named which creates an
/// executor and adds the marked function as a task. If any enabled
/// Cntrlr features require background tasks (such as USB), those
/// tasks will also be added to the executor.
#[proc_macro_attribute]
pub fn entry(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;

    let main_is_valid = sig.asyncness.is_some()
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.inputs.is_empty()
        && match sig.output {
            ReturnType::Type(_, ref typ) => matches!(**typ, Type::Never(_)),
            ReturnType::Default => false,
        };
    if !main_is_valid {
        return ParseError::new(
            input_fn.sig.span(),
            format!(
                "Cntrlr entry function must be of the form `async fn {}() -> !`",
                fn_name
            ),
        )
        .to_compile_error()
        .into();
    }

    quote!(
        #[export_name = "__cntrlr_main"]
        // This is flagged as unsafe just in case the input_fn is
        // unsafe, so that we can call it.
        pub unsafe extern "C" fn #fn_name() -> ! {
            #input_fn

            let mut executor =  ::cntrlr::task::Executor::new();
            executor.add_task(#fn_name());
            // TODO: Add tasks for device drivers as needed.
            executor.run()
        }
    )
    .into()
}

/// Override the default task initialization
///
/// This allows you control of Cntrlr application startup, including
/// whether or not to use an async executor and which tasks are added
/// to it.
#[proc_macro_attribute]
pub fn raw_entry(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;

    let main_is_valid = sig.asyncness.is_none()
        && match sig.abi {
            None => false,
            Some(ref abi) => match abi.name {
                None => true,
                Some(ref abi) => abi.value() == "C",
            },
        }
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.inputs.is_empty()
        && match sig.output {
            ReturnType::Type(_, ref typ) => matches!(**typ, Type::Never(_)),
            ReturnType::Default => false,
        };
    if !main_is_valid {
        return ParseError::new(
            input_fn.sig.span(),
            format!(
                "Cntrlr entry function must be of the form `extern \"C\" fn {}() -> !`",
                fn_name
            ),
        )
        .to_compile_error()
        .into();
    }

    quote!(
        #[export_name = "__cntrlr_main"]
        #input_fn
    )
    .into()
}

/// Override the default reset vector
///
/// When you implement the reset vector, you are responsible for all
/// chip and runtime initialization, including such things as loading
/// the data segment and clearing bss. You probably don't want to do
/// this. See [`macro@raw_entry`] if you want to take over after minimal
/// board init has been completed.
#[proc_macro_attribute]
pub fn reset(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;

    let reset_is_valid = sig.asyncness.is_none()
        && match sig.abi {
            None => false,
            Some(ref abi) => match abi.name {
                None => true,
                Some(ref abi) => abi.value() == "C",
            },
        }
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.inputs.is_empty()
        && match sig.output {
            ReturnType::Type(_, ref typ) => matches!(**typ, Type::Never(_)),
            ReturnType::Default => false,
        };
    if !reset_is_valid {
        return ParseError::new(
            input_fn.sig.span(),
            format!(
                "Cntrlr reset function must be of the form `extern \"C\" fn {}() -> !`",
                fn_name
            ),
        )
        .to_compile_error()
        .into();
    }

    quote!(
        #[link_section = ".__CNTRLR_START"]
        #[export_name = "__cntrlr_reset"]
        #input_fn
    )
    .into()
}
