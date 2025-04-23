#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str, punctuated::Punctuated, Error, Expr, LitInt, LitStr,
    Token,
};
/// The default buffer size for the logger.
const DEFAULT_BUFFER_SIZE: &str = "200";
/// Represents the input arguments to the `log!` macro.
struct LogArgs {
    /// The length of the buffer to use for the logger.
    ///
    /// This does not have effect when the literal `str` does
    /// not have value placeholders.
    buffer_len: LitInt,
    /// The literal formatting string passed to the macro.
    ///
    /// The `str` might have value placeholders. While this is
    /// not a requirement, the number of placeholders must
    /// match the number of args.
    format_string: LitStr,
    /// The arguments passed to the macro.
    ///
    /// The arguments represent the values to replace the
    /// placeholders on the format `str`. Valid values must implement
    /// the [`Log`] trait.
    args: Punctuated<Expr, ::syn::token::Comma>,
}
impl Parse for LogArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let buffer_len = if input.peek(LitInt) {
            let literal = input.parse()?;
            input.parse::<::syn::token::Comma>()?;
            literal
        } else {
            parse_str::<LitInt>(DEFAULT_BUFFER_SIZE)?
        };
        let format_string = input.parse()?;
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<::syn::token::Comma>()?;
            Punctuated::parse_terminated(input)?
        };
        Ok(LogArgs {
            buffer_len,
            format_string,
            args,
        })
    }
}
/// Companion `log!` macro for `pinocchio-log`.
///
/// The macro automates the creation of a `Logger` object to log a message.
/// It support a limited subset of the [`format!`](https://doc.rust-lang.org/std/fmt/) syntax.
/// The macro parses the format string at compile time and generates the calls to a `Logger`
/// object to generate the corresponding formatted message.
///
/// # Arguments
///
/// - `buffer_len`: The length of the buffer to use for the logger (default to `200`). This is an optional argument.
/// - `format_string`: The literal string to log. This string can contain placeholders `{}` to be replaced by the arguments.
/// - `args`: The arguments to replace the placeholders in the format string. The arguments must implement the `Log` trait.
#[proc_macro]
pub fn log(input: TokenStream) -> TokenStream {
    let LogArgs { buffer_len, format_string, args } = match ::syn::parse_macro_input::parse::<
        LogArgs,
    >(input) {
        ::syn::__private::Ok(data) => data,
        ::syn::__private::Err(err) => {
            return ::syn::__private::TokenStream::from(err.to_compile_error());
        }
    };
    let parsed_string = format_string.value();
    let placeholder_regex = Regex::new(r"\{.*?\}").unwrap();
    let placeholders: Vec<_> = placeholder_regex
        .find_iter(&parsed_string)
        .map(|m| m.as_str())
        .collect();
    if placeholders.len() != args.len() {
        let arg_message = if args.is_empty() {
            "but no arguments were given".to_string()
        } else {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!(
                        "but there is {0} {1}",
                        args.len(),
                        if args.len() == 1 { "argument" } else { "arguments" },
                    ),
                );
                res
            })
        };
        return Error::new_spanned(
                format_string,
                ::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "{0} positional arguments in format string, {1}",
                            placeholders.len(),
                            arg_message,
                        ),
                    );
                    res
                }),
            )
            .to_compile_error()
            .into();
    }
    if !placeholders.is_empty() {
        let mut replaced_parts = Vec::new();
        let parts: Vec<&str> = placeholder_regex.split(&parsed_string).collect();
        let part_iter = parts.iter();
        let mut arg_iter = args.iter();
        let mut ph_iter = placeholders.iter();
        for part in part_iter {
            if !part.is_empty() {
                replaced_parts
                    .push({
                        let mut _s = ::quote::__private::TokenStream::new();
                        ::quote::__private::push_ident(&mut _s, "logger");
                        ::quote::__private::push_dot(&mut _s);
                        ::quote::__private::push_ident(&mut _s, "append");
                        ::quote::__private::push_group(
                            &mut _s,
                            ::quote::__private::Delimiter::Parenthesis,
                            {
                                let mut _s = ::quote::__private::TokenStream::new();
                                ::quote::ToTokens::to_tokens(&part, &mut _s);
                                _s
                            },
                        );
                        _s
                    });
            }
            if let Some(arg) = arg_iter.next() {
                let placeholder = ph_iter.next().unwrap();
                match *placeholder {
                    "{}" => {
                        replaced_parts
                            .push({
                                let mut _s = ::quote::__private::TokenStream::new();
                                ::quote::__private::push_ident(&mut _s, "logger");
                                ::quote::__private::push_dot(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "append");
                                ::quote::__private::push_group(
                                    &mut _s,
                                    ::quote::__private::Delimiter::Parenthesis,
                                    {
                                        let mut _s = ::quote::__private::TokenStream::new();
                                        ::quote::ToTokens::to_tokens(&arg, &mut _s);
                                        _s
                                    },
                                );
                                _s
                            });
                    }
                    value if value.starts_with("{:.") => {
                        let precision = if let Ok(precision) = value[3..value.len() - 1]
                            .parse::<u8>()
                        {
                            precision
                        } else {
                            return Error::new_spanned(
                                    format_string,
                                    ::alloc::__export::must_use({
                                        let res = ::alloc::fmt::format(
                                            format_args!("invalid precision format: {0}", value),
                                        );
                                        res
                                    }),
                                )
                                .to_compile_error()
                                .into();
                        };
                        replaced_parts
                            .push({
                                let mut _s = ::quote::__private::TokenStream::new();
                                ::quote::__private::push_ident(&mut _s, "logger");
                                ::quote::__private::push_dot(&mut _s);
                                ::quote::__private::push_ident(&mut _s, "append_with_args");
                                ::quote::__private::push_group(
                                    &mut _s,
                                    ::quote::__private::Delimiter::Parenthesis,
                                    {
                                        let mut _s = ::quote::__private::TokenStream::new();
                                        ::quote::ToTokens::to_tokens(&arg, &mut _s);
                                        ::quote::__private::push_comma(&mut _s);
                                        ::quote::__private::push_and(&mut _s);
                                        ::quote::__private::push_group(
                                            &mut _s,
                                            ::quote::__private::Delimiter::Bracket,
                                            {
                                                let mut _s = ::quote::__private::TokenStream::new();
                                                ::quote::__private::push_ident(&mut _s, "pinocchio_log");
                                                ::quote::__private::push_colon2(&mut _s);
                                                ::quote::__private::push_ident(&mut _s, "logger");
                                                ::quote::__private::push_colon2(&mut _s);
                                                ::quote::__private::push_ident(&mut _s, "Argument");
                                                ::quote::__private::push_colon2(&mut _s);
                                                ::quote::__private::push_ident(&mut _s, "Precision");
                                                ::quote::__private::push_group(
                                                    &mut _s,
                                                    ::quote::__private::Delimiter::Parenthesis,
                                                    {
                                                        let mut _s = ::quote::__private::TokenStream::new();
                                                        ::quote::ToTokens::to_tokens(&precision, &mut _s);
                                                        _s
                                                    },
                                                );
                                                _s
                                            },
                                        );
                                        _s
                                    },
                                );
                                _s
                            });
                    }
                    value if value.starts_with("{:<.") || value.starts_with("{:>.") => {
                        let size = if let Ok(size) = value[4..value.len() - 1]
                            .parse::<usize>()
                        {
                            size
                        } else {
                            return Error::new_spanned(
                                    format_string,
                                    ::alloc::__export::must_use({
                                        let res = ::alloc::fmt::format(
                                            format_args!("invalid truncate size format: {0}", value),
                                        );
                                        res
                                    }),
                                )
                                .to_compile_error()
                                .into();
                        };
                        match value.chars().nth(2) {
                            Some('<') => {
                                replaced_parts
                                    .push({
                                        let mut _s = ::quote::__private::TokenStream::new();
                                        ::quote::__private::push_ident(&mut _s, "logger");
                                        ::quote::__private::push_dot(&mut _s);
                                        ::quote::__private::push_ident(&mut _s, "append_with_args");
                                        ::quote::__private::push_group(
                                            &mut _s,
                                            ::quote::__private::Delimiter::Parenthesis,
                                            {
                                                let mut _s = ::quote::__private::TokenStream::new();
                                                ::quote::ToTokens::to_tokens(&arg, &mut _s);
                                                ::quote::__private::push_comma(&mut _s);
                                                ::quote::__private::push_and(&mut _s);
                                                ::quote::__private::push_group(
                                                    &mut _s,
                                                    ::quote::__private::Delimiter::Bracket,
                                                    {
                                                        let mut _s = ::quote::__private::TokenStream::new();
                                                        ::quote::__private::push_ident(&mut _s, "pinocchio_log");
                                                        ::quote::__private::push_colon2(&mut _s);
                                                        ::quote::__private::push_ident(&mut _s, "logger");
                                                        ::quote::__private::push_colon2(&mut _s);
                                                        ::quote::__private::push_ident(&mut _s, "Argument");
                                                        ::quote::__private::push_colon2(&mut _s);
                                                        ::quote::__private::push_ident(&mut _s, "TruncateStart");
                                                        ::quote::__private::push_group(
                                                            &mut _s,
                                                            ::quote::__private::Delimiter::Parenthesis,
                                                            {
                                                                let mut _s = ::quote::__private::TokenStream::new();
                                                                ::quote::ToTokens::to_tokens(&size, &mut _s);
                                                                _s
                                                            },
                                                        );
                                                        _s
                                                    },
                                                );
                                                _s
                                            },
                                        );
                                        _s
                                    });
                            }
                            Some('>') => {
                                replaced_parts
                                    .push({
                                        let mut _s = ::quote::__private::TokenStream::new();
                                        ::quote::__private::push_ident(&mut _s, "logger");
                                        ::quote::__private::push_dot(&mut _s);
                                        ::quote::__private::push_ident(&mut _s, "append_with_args");
                                        ::quote::__private::push_group(
                                            &mut _s,
                                            ::quote::__private::Delimiter::Parenthesis,
                                            {
                                                let mut _s = ::quote::__private::TokenStream::new();
                                                ::quote::ToTokens::to_tokens(&arg, &mut _s);
                                                ::quote::__private::push_comma(&mut _s);
                                                ::quote::__private::push_and(&mut _s);
                                                ::quote::__private::push_group(
                                                    &mut _s,
                                                    ::quote::__private::Delimiter::Bracket,
                                                    {
                                                        let mut _s = ::quote::__private::TokenStream::new();
                                                        ::quote::__private::push_ident(&mut _s, "pinocchio_log");
                                                        ::quote::__private::push_colon2(&mut _s);
                                                        ::quote::__private::push_ident(&mut _s, "logger");
                                                        ::quote::__private::push_colon2(&mut _s);
                                                        ::quote::__private::push_ident(&mut _s, "Argument");
                                                        ::quote::__private::push_colon2(&mut _s);
                                                        ::quote::__private::push_ident(&mut _s, "TruncateEnd");
                                                        ::quote::__private::push_group(
                                                            &mut _s,
                                                            ::quote::__private::Delimiter::Parenthesis,
                                                            {
                                                                let mut _s = ::quote::__private::TokenStream::new();
                                                                ::quote::ToTokens::to_tokens(&size, &mut _s);
                                                                _s
                                                            },
                                                        );
                                                        _s
                                                    },
                                                );
                                                _s
                                            },
                                        );
                                        _s
                                    });
                            }
                            _ => {
                                return Error::new_spanned(
                                        format_string,
                                        ::alloc::__export::must_use({
                                            let res = ::alloc::fmt::format(
                                                format_args!("invalid truncate format: {0}", value),
                                            );
                                            res
                                        }),
                                    )
                                    .to_compile_error()
                                    .into();
                            }
                        }
                    }
                    _ => {
                        return Error::new_spanned(
                                format_string,
                                ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("invalid placeholder: {0}", placeholder),
                                    );
                                    res
                                }),
                            )
                            .to_compile_error()
                            .into();
                    }
                }
            }
        }
        TokenStream::from({
            let mut _s = ::quote::__private::TokenStream::new();
            ::quote::__private::push_group(
                &mut _s,
                ::quote::__private::Delimiter::Brace,
                {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::__private::push_ident(&mut _s, "let");
                    ::quote::__private::push_ident(&mut _s, "mut");
                    ::quote::__private::push_ident(&mut _s, "logger");
                    ::quote::__private::push_eq(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "pinocchio_log");
                    ::quote::__private::push_colon2(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "logger");
                    ::quote::__private::push_colon2(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "Logger");
                    ::quote::__private::push_colon2(&mut _s);
                    ::quote::__private::push_lt(&mut _s);
                    ::quote::ToTokens::to_tokens(&buffer_len, &mut _s);
                    ::quote::__private::push_gt(&mut _s);
                    ::quote::__private::push_colon2(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "default");
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Parenthesis,
                        ::quote::__private::TokenStream::new(),
                    );
                    ::quote::__private::push_semi(&mut _s);
                    {
                        use ::quote::__private::ext::*;
                        let has_iter = ::quote::__private::ThereIsNoIteratorInRepetition;
                        #[allow(unused_mut)]
                        let (mut replaced_parts, i) = replaced_parts.quote_into_iter();
                        let has_iter = has_iter | i;
                        let _: ::quote::__private::HasIterator = has_iter;
                        while true {
                            let replaced_parts = match replaced_parts.next() {
                                Some(_x) => ::quote::__private::RepInterp(_x),
                                None => break,
                            };
                            ::quote::ToTokens::to_tokens(&replaced_parts, &mut _s);
                            ::quote::__private::push_semi(&mut _s);
                        }
                    }
                    ::quote::__private::push_ident(&mut _s, "logger");
                    ::quote::__private::push_dot(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "log");
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Parenthesis,
                        ::quote::__private::TokenStream::new(),
                    );
                    ::quote::__private::push_semi(&mut _s);
                    _s
                },
            );
            _s
        })
    } else {
        TokenStream::from({
            let mut _s = ::quote::__private::TokenStream::new();
            ::quote::__private::push_ident(&mut _s, "pinocchio_log");
            ::quote::__private::push_colon2(&mut _s);
            ::quote::__private::push_ident(&mut _s, "logger");
            ::quote::__private::push_colon2(&mut _s);
            ::quote::__private::push_ident(&mut _s, "log_message");
            ::quote::__private::push_group(
                &mut _s,
                ::quote::__private::Delimiter::Parenthesis,
                {
                    let mut _s = ::quote::__private::TokenStream::new();
                    ::quote::ToTokens::to_tokens(&format_string, &mut _s);
                    ::quote::__private::push_dot(&mut _s);
                    ::quote::__private::push_ident(&mut _s, "as_bytes");
                    ::quote::__private::push_group(
                        &mut _s,
                        ::quote::__private::Delimiter::Parenthesis,
                        ::quote::__private::TokenStream::new(),
                    );
                    _s
                },
            );
            ::quote::__private::push_semi(&mut _s);
            _s
        })
    }
}
const _: () = {
    extern crate proc_macro;
    #[rustc_proc_macro_decls]
    #[used]
    #[allow(deprecated)]
    static _DECLS: &[proc_macro::bridge::client::ProcMacro] = &[
        proc_macro::bridge::client::ProcMacro::bang("log", log),
    ];
};
