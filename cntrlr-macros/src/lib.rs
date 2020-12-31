use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Error as ParseError, Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Ident, ItemFn, ItemUse, ReturnType, Type,
};

struct Boards {
    boards: Punctuated<Ident, Comma>,
}

impl Parse for Boards {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Boards {
            boards: input.parse_terminated(Ident::parse)?,
        })
    }
}

#[proc_macro_attribute]
pub fn prelude_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_use = parse_macro_input!(input as ItemUse);
    let boards = parse_macro_input!(args as Boards);

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

#[proc_macro_attribute]
pub fn board_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;
    let boards = parse_macro_input!(args as Boards);
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

    let impls = boards
        .boards
        .iter()
        .skip(1)
        .map(|board| {
            let board_name = format!("{}", board);
            quote!(
            #[cfg(board = #board_name)]
            {
                crate::hw::board::#board::#module::#fn_name()
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
/// This macro defines a startup routine named `__cntrlr_main`, which
/// creates an executor and adds the marked function as a task. If any
/// enabled Cntrlr features require background tasks (such as USB),
/// those tasks will also be added to the executor.
#[proc_macro_attribute]
pub fn entry(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;

    let main_is_valid = sig.asyncness.is_some()
        && sig.unsafety.is_none()
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
                "Cntrlr main function must be of the form `async fn {}() -> !`",
                fn_name
            ),
        )
        .to_compile_error()
        .into();
    }

    quote!(
        #[no_mangle]
        pub unsafe extern "C" fn __cntrlr_main() -> ! {
            let mut executor =  ::cntrlr::task::Executor::new();

            executor.add_task(#fn_name());
            // TODO: Add tasks for device drivers as needed.
            executor.run()
        }

        #input_fn
    )
    .into()
}

/// Override the default reset vector
///
/// When you implement the reset vector, you are responsible for all
/// chip and runtime initialization, including such things as loading
/// the data segment and clearing bss. You probably don't want to do
/// this.
#[proc_macro_attribute]
pub fn reset(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let sig = &input_fn.sig;
    let fn_name = &sig.ident;

    let reset_is_valid = sig.asyncness.is_none()
        && sig.unsafety.is_some()
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
                "Cntrlr reset function must be of the form `unsafe extern \"C\" fn {}() -> !`",
                fn_name
            ),
        )
        .to_compile_error()
        .into();
    }

    quote!(
        #[link_section = ".__CNTRLR_START"]
        #[no_mangle]
        pub unsafe extern "C" fn __cntrlr_reset() -> ! {
            #fn_name()
        }

        #input_fn
    )
    .into()
}
