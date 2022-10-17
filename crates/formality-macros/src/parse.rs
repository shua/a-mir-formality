extern crate proc_macro;

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute};

use self::spec::{FieldMode, FormalitySpec};

mod spec;

pub(crate) fn derive_parse(s: synstructure::Structure) -> TokenStream {
    if let syn::Data::Union(v) = &s.ast().data {
        return syn::Error::new(v.union_token.span, "unions are not supported")
            .into_compile_error();
    }

    let mut stream = TokenStream::new();

    if s.variants().len() == 1 {
        stream.extend(parse_variant(&s.variants()[0]));
    } else {
        stream.extend(quote! {
            let mut __results = vec![];
        });
        for variant in s.variants() {
            let v = parse_variant(variant);
            stream.extend(quote! {
                __results.push({
                   parse::try_parse(|| { #v })
                });
            });
        }
        stream.extend(quote! {
            if __results.len() > 1 {
                panic!("ambiguous grammer, parsed: {:?}", __results);
            }
            __results.pop()
        });
    }

    s.gen_impl(quote! {
        use crate::derive_links::{parse};

        gen impl parse::Parse for @Self {
            fn parse<'t>(scope: &Scope, text: &'t str) -> Option<(Self, &'t str)>
            {
                #stream
            }
        }
    })
}

fn parse_variant(variant: &synstructure::VariantInfo) -> TokenStream {
    let ast = variant.ast();
    if let Some(attr) = get_grammar_attr(ast.attrs) {
        return match attr {
            Ok(spec) => parse_variant_with_attr(variant, &spec),
            Err(err) => err.into_compile_error(),
        };
    }

    // If no `#[formality(...)]` attribute is provided, then we expect
    // one of two situations:
    //
    // `Foo` -- just parse a keyword
    // `Foo(Bar)` -- single field, just parse that and wrap it

    match variant.bindings() {
        [] => {
            let literal = Literal::string(&ast.ident.to_string().to_lowercase());
            let construct = variant.construct(|_, _| quote! {});
            quote! {
                let text = parse::expect_keyword(text, #literal)?;
                Some((#construct, text))
            }
        }

        [_] => variant.construct(|field, _| {
            let field_ty = &field.ty;
            quote!(<#field_ty as Parse > :: parse(scope, text))
        }),

        _ => syn::Error::new(
            ast.ident.span(),
            "formality attribute required to guide parsing",
        )
        .into_compile_error(),
    }
}

/// When a type is given a formality attribute, we use that to guide parsing:
///
/// ```rust,ignore
/// #[formality(impl $data)]
/// struct Impl {
///     data: Binder<ImplBound>
/// }
///
/// #[formality($trait_id < $*tys > for $self_ty where $,where_clauses { $*trait_items })]
/// struct ImplBound {
///     trait_id: TraitId,
///     tys: Vec<Ty>,
///     self_ty: Ty,
///     where_clauses: Vec<WhereClause>,
/// }
/// ```
fn parse_variant_with_attr(
    variant: &synstructure::VariantInfo,
    spec: &FormalitySpec,
) -> TokenStream {
    let mut stream = TokenStream::new();

    for binding in variant.bindings() {
        if binding.ast().ident.is_none() {
            return syn::Error::new(binding.ast().ty.span(), "anonymous bindings not supported")
                .into_compile_error();
        }
    }

    for op in &spec.ops {
        stream.extend(match op {
            spec::FormalitySpecOp::Field {
                name,
                mode: FieldMode::Single,
            } => {
                quote_spanned! {
                    name.span() => let (#name, text) = parse::Parse::parse(scope, text)?;
                }
            }

            spec::FormalitySpecOp::Field {
                name,
                mode: FieldMode::Many,
            } => {
                quote_spanned! {
                    name.span() => let (#name, text) = parse::Parse::parse_many(scope, text)?;
                }
            }

            spec::FormalitySpecOp::Field {
                name,
                mode: FieldMode::Comma,
            } => {
                quote_spanned! {
                    name.span() => let (#name, text) = parse::Parse::parse_comma(scope, text)?;
                }
            }

            spec::FormalitySpecOp::Keyword { ident } => {
                let literal = as_literal(ident);
                quote_spanned!(ident.span() => let text = parse::expect_keyword(text, #literal)?;)
            }

            spec::FormalitySpecOp::Char { punct } => {
                let literal = Literal::character(punct.as_char());
                quote_spanned!(punct.span() => let text = parse::expect(#literal)?;)
            }

            spec::FormalitySpecOp::Text { text } => {
                let literal = Literal::string(text);
                quote!(let text = parse::expect_str(#literal)?;)
            }
        });
    }

    let c = variant.construct(|field, _| match &field.ident {
        Some(field_name) => field_name.clone(),
        None => unreachable!(),
    });

    stream.extend(quote! {
        Some((#c, text))
    });

    stream
}

fn get_grammar_attr(attrs: &[Attribute]) -> Option<syn::Result<FormalitySpec>> {
    let attr = attrs.iter().find(|a| a.path.is_ident("grammar"))?;
    Some(attr.parse_args())
}

fn as_literal(ident: &Ident) -> Literal {
    Literal::string(&ident.to_string())
}

#[test]
fn test_enum() {
    synstructure::test_derive! {
            derive_parse {
                enum A {
                    B(B),
                    C(C),
                }
            }
            expands to {
                # [
        allow (
            non_upper_case_globals)
        ]
    const _DERIVE_parse_Parse_FOR_A : (
        )
    = {
        use crate :: derive_links :: {
            parse }
        ;
        impl parse :: Parse for A {
            fn parse < 't > (
                scope : & Scope , text : & 't str)
            -> Option < (
                Self , & 't str)
            > {
                let mut __results = vec ! [
                    ]
                ;
                __results . push (
                    {
                        parse :: try_parse (
                            || {
                                A :: B (
                                    < B as Parse > :: parse (
                                        scope , text)
                                    ,)
                                }
                            )
                        }
                    )
                ;
                __results . push (
                    {
                        parse :: try_parse (
                            || {
                                A :: C (
                                    < C as Parse > :: parse (
                                        scope , text)
                                    ,)
                                }
                            )
                        }
                    )
                ;
                if __results . len (
                    )
                > 1 {
                    panic ! (
                        "ambiguous grammer, parsed: {:?}" , __results)
                    ;
                    }
                __results . pop (
                    )
                }
            }
        }
    ;
        }
        no_build
    }
}

#[test]
fn test_enum_grammar() {
    synstructure::test_derive! {
            derive_parse {
                #[grammar(impl $name < $*ty > for $ty_self where $,wc { $*items })]
                struct Impl {
                    name: Id,
                    ty: Vec<Ty>,
                    ty_self: Ty,
                    wc: Vec<WhereClause>,
                    items: Vec<Item>,
                }
            }
            expands to {
                # [
        allow (
            non_upper_case_globals)
        ]
    const _DERIVE_parse_Parse_FOR_Impl : (
        )
    = {
        use crate :: derive_links :: {
            parse }
        ;
        impl parse :: Parse for Impl {
            fn parse < 't > (
                scope : & Scope , text : & 't str)
            -> Option < (
                Self , & 't str)
            > {
                let text = parse :: expect_keyword (
                    text , "impl")
                ? ;
                let (
                    name , text)
                = parse :: Parse :: parse (
                    scope , text)
                ? ;
                let text = parse :: expect (
                    '<')
                ? ;
                let (
                    ty , text)
                = parse :: Parse :: parse_many (
                    scope , text)
                ? ;
                let text = parse :: expect (
                    '>')
                ? ;
                let text = parse :: expect_keyword (
                    text , "for")
                ? ;
                let (
                    ty_self , text)
                = parse :: Parse :: parse (
                    scope , text)
                ? ;
                let text = parse :: expect_keyword (
                    text , "where")
                ? ;
                let (
                    wc , text)
                = parse :: Parse :: parse_comma (
                    scope , text)
                ? ;
                let text = parse :: expect_str (
                    "[")
                    ? ;
                    let (
                        items , text)
                    = parse :: Parse :: parse_many (
                        scope , text)
                    ? ;
                    let text = parse :: expect_str (
                        "]")
                ? ;
                Some (
                    (
                        Impl {
                            name : name , ty : ty , ty_self : ty_self , wc : wc , items : items , }
                        , text)
                    )
                }
            }
        }
    ;
            }
            no_build
        }
}
