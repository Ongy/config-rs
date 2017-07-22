extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use std::collections::HashSet;

fn impl_get_name(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    tok.append("fn get_name() -> &'static str { ");
    /* TODO: Figure out, if this can be canonicalized to package/full::path or similar to avoid
     * naming collissions that hide some module type
     */
    tok.append(quote!{ return stringify!(#name); });
    tok.append("}"); /* Close get_name() */
}

fn append_fields<'a, I>(fields: I, tok: &mut quote::Tokens, others: &mut HashSet<&'a syn::Ty>)
    where I: std::iter::Iterator<Item=&'a syn::Field> {
    let mut first = true;

    for ref field in fields {
        if first {
            first = false;
        } else {
            tok.append(quote!{ret.push_str(", ");});
        }
        let ty = &field.ty;

        if let Some(ref id) = field.ident {
            tok.append(quote!{ret.push_str(stringify!(#id)); ret.push_str(": ");});
        }

        tok.append(quote!{ret.push_str(stringify!(#ty));});
        others.insert(ty);
    }
}

fn impl_parse_named<'a, I>(fields: I, tok: &mut quote::Tokens)
    where I: std::iter::Iterator<Item=&'a syn::Field> + std::clone::Clone {

    tok.append(quote!{provider.consume_char('{', fun)?;});
    for ref field in fields.clone() {
        let name = match field.ident {
            Some(ref x) => x,
            None => panic!("Encountered unnamed field while trying to derive named field parsing")
        };
        let ty = &field.ty;

        tok.append(quote!(let mut #name:ParseTmp<#ty> = ParseTmp::Empty;));
    }

    tok.append("loop {");
    tok.append(quote!{
        if provider.peek_char() == Some('}') {
            provider.consume(1).unwrap();
            break;
        }

        let nxt = match provider.get_next() {
            Some(x) => x,
            None => {
                fun("Reached end of file while trying to parse named values".to_string());
                return Err(ParseError::Final);
            }
        };
    });

    for ref field in fields {
        let name = match field.ident {
            Some(ref x) => x,
            None => panic!("Encountered unnamed field while trying to derive named field parsing")
        };
        let ty = &field.ty;

        tok.append("if ");
        tok.append(quote!{nxt.starts_with(stringify!(#name))});
        tok.append("{");
        tok.append(quote!{
            provider.consume(stringify!(#name).len()).unwrap();
            provider.consume_char(':', fun)?;
            #name.push_found(#ty::parse_from(provider, fun), fun)?;

            if provider.peek_char() == Some(',') {
                provider.consume(1).unwrap();
            }
        });
        tok.append("}");
    }

    tok.append("}");
}

fn impl_parse_ordered<'a, I>(fields: I, tok: &mut quote::Tokens)
    where I: std::iter::Iterator<Item=&'a syn::Field> {

    tok.append(quote!{provider.consume_char('(', fun)?;});
    for (index, ref field) in fields.enumerate() {
        if index > 0 {
            tok.append(quote!{provider.consume_char(',', fun)?;});
        }
        let ty = &field.ty;

        tok.append(format!("let var{} =", index));
        
        tok.append(quote!{ #ty::parse_from(provider, fun)?; });
    }
    tok.append(quote!{provider.consume_char(')', fun)?;});
}

fn impl_get_format(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    let mut others = HashSet::new();

    tok.append("#[allow(unused_variables)]"); /* Allow unused set argument */
    tok.append("fn get_format(set: &mut ::std::collections::HashSet<String>) -> String {");
    tok.append("let mut ret = String::new();");

    tok.append(format!("ret.push_str(\"{}: \");", name));

    match ast.body {
        /* Handle Enums */
        syn::Body::Enum(ref vars) => {
            let mut first = true;
            for ref var in vars {
                let vname = &var.ident;
                if first {
                    first = false
                } else {
                    tok.append(quote!{ret.push_str(" | ");});
                }

                tok.append(quote!{ret.push_str(stringify!(#vname));});

                match var.data {
                    syn::VariantData::Unit => {},
                    syn::VariantData::Tuple(ref fields) => {
                        tok.append(quote!{ret.push_str("(");});
                        append_fields(fields.iter(), tok, &mut others);
                        tok.append(quote!{ret.push_str(")");});
                    },
                    syn::VariantData::Struct(ref fields) => {
                        tok.append(quote!{ret.push_str("{");});
                        append_fields(fields.iter(), tok, &mut others);
                        tok.append(quote!{ret.push_str("}");});
                    },
                }

            }
        },
        /* Handle structs */
        syn::Body::Struct(ref data) => {

            match data {
                &syn::VariantData::Unit => {
                    panic!("Error while creating config scheme for {}. Can't do anything nice with Unit structs", name);
                },
                &syn::VariantData::Tuple(ref fields) => {
                    tok.append(quote!{ret.push_str("(");});
                    append_fields(fields.iter(), tok, &mut others);
                    tok.append(quote!{ret.push_str(")");});
                },
                &syn::VariantData::Struct(ref fields) => {
                    tok.append(quote!{ret.push_str("{");});
                    append_fields(fields.iter(), tok, &mut others);
                    tok.append(quote!{ret.push_str("}");});
                },
            }
        }
    }

    tok.append("ret.push_str(\"\\n\");");

    /* Append other types format, so the entire used type tree will be displayed */
    for other in others {
        tok.append(quote!{
            { /* This will be a block, to avoid naming collisions */
                /* Check if we already appended the other type somewhere*/
                let name = String::from(#other::get_name());
                if !set.contains(&name) {
                    /* If we didn't, insert it into the list of printed types and append it */
                    set.insert(name);

                    ret.push_str(#other::get_format(set).as_str());
                }
            } /* Close the scoping block */
        });
    }

    tok.append("return ret; } "); /* Close print_format */
}

fn impl_parse_from(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    tok.append(quote!{#[allow(unused_variables, unreachable_code)]
        fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<Self, ParseError>
            where I: ::std::iter::Iterator<Item=(usize, String)>,
                  F: FnMut(String)
    });
    tok.append("{");
    tok.append(quote!{let nxt = match provider.get_next() {
            Some(x) => x,
            None => {
                fun("Was at end of file.".to_string());
                return Err(ParseError::Final);
            }
        };
    });

    match ast.body {
        /* Handle Enums */
        syn::Body::Enum(ref vars) => {
            for ref var in vars {
                let vname = &var.ident;

                match var.data {
                    syn::VariantData::Unit => {
                        tok.append(quote!{
                            if nxt.starts_with(stringify!(#vname)) {
                                provider.consume(stringify!(#vname).len()).unwrap();
                                return Ok(#name::#vname);
                            }
                        });
                    },
                    syn::VariantData::Tuple(ref fields) => {
                        tok.append(quote!{ if nxt.starts_with(stringify!(#vname))});
                        tok.append("{");
                        tok.append(quote!{ provider.consume(stringify!(#vname).len()).unwrap();});
                        impl_parse_ordered(fields.iter(), tok);
                        tok.append("return Ok(");
                        tok.append(quote!{#name::#vname});
                        tok.append("(");

                        for (i, _) in fields.iter().enumerate() {
                            if i > 0 {
                                tok.append(",");
                            }
                            tok.append(format!("var{}", i));
                        }

                        tok.append("));");
                        tok.append("}");
                    },
                    syn::VariantData::Struct(ref fields) => {
                        tok.append(quote!{ if nxt.starts_with(stringify!(#vname))});
                        tok.append("{");
                        tok.append(quote!{ provider.consume(stringify!(#vname).len()).unwrap();});
                        impl_parse_named(fields.iter(), tok);

                        impl_parse_named(fields.iter(), tok);
                        tok.append("return Ok(");
                        tok.append(quote!{#name::#vname});
                        tok.append("{");

                        for (i, ref field) in fields.iter().enumerate() {
                            if i > 0 {
                                tok.append(",");
                            }

                            let name = match field.ident {
                                Some(ref x) => x,
                                None => panic!("Encountered unnamed field while trying to derive named field parsing")
                            };

                            tok.append(quote!{ #name: #name.get_value()? });

                        }

                        tok.append("});");
                        tok.append("}");
                    },
                }
            }
        },
        /* Handle structs */
        syn::Body::Struct(ref data) => {

            match data {
                &syn::VariantData::Unit => {
                },
                &syn::VariantData::Tuple(ref fields) => {
                    impl_parse_ordered(fields.iter(), tok);
                    tok.append("return Ok(");
                    tok.append(quote!{#name});
                    tok.append("(");

                    for (i, _) in fields.iter().enumerate() {
                        if i > 0 {
                            tok.append(",");
                        }
                        tok.append(format!("var{}", i));
                    }

                    tok.append("));");
                },
                &syn::VariantData::Struct(ref fields) => {
                    impl_parse_named(fields.iter(), tok);
                    tok.append("return Ok(");
                    tok.append(quote!{#name});
                    tok.append("{");

                    for (i, ref field) in fields.iter().enumerate() {
                        if i > 0 {
                            tok.append(",");
                        }

                        let name = match field.ident {
                            Some(ref x) => x,
                            None => panic!("Encountered unnamed field while trying to derive named field parsing")
                        };

                        tok.append(quote!{ #name: #name.get_value()? });

                    }

                    tok.append("});");
                },
            }
        }
    }

    tok.append(quote!{
        fun(format!("Tried to parse {}, found '{}' which I couldn't handle", stringify!(#name), nxt));
        return Err(ParseError::Final);
    });
    tok.append("}");
}

fn impl_derive_config_able(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;

    let mut start = quote::Tokens::new();
    start.append("impl ConfigAble for ");
    start.append(format!("{} {{", name));

    impl_get_format(ast, &mut start);
    impl_get_name(ast, &mut start);
    impl_parse_from(ast, &mut start);

    start.append("}"); /* Close impl */

    return start;
}

#[proc_macro_derive(ConfigAble)]
pub fn print_format(input: TokenStream) -> TokenStream {
    let s = input.to_string();

    let ast = syn::parse_derive_input(&s).unwrap();

    let gen = impl_derive_config_able(&ast);

    return gen.parse().unwrap();
}

#[cfg(test)]
mod test {

}
