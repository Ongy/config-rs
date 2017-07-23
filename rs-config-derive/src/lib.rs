extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use std::collections::HashSet;

fn get_attrs(field: &syn::Field) -> Option<&Vec<syn::NestedMetaItem>> {
    for ref attr in &field.attrs {
        match attr.value {
            syn::MetaItem::List(ref id, ref items) => {
                if id == "ConfigAttrs" {
                    return Some(items);
                }
            },
            _ => {},
        }
    }

    return None;
}

fn find_attr_lit<'a>(name: &str, attrs: &'a Vec<syn::NestedMetaItem>) -> Option<&'a syn::Lit> {
    for attr in attrs {
        match attr {
            &syn::NestedMetaItem::MetaItem(ref val) => {
                match val {
                    &syn::MetaItem::NameValue(ref id, ref lit) => {
                        if id == name {
                            return Some(lit);
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }

    return None;
}

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
            tok.append(quote!{fun(", ");});
        }
        let ty = &field.ty;

        if let Some(ref id) = field.ident {
            tok.append(quote!{fun(stringify!(#id)); fun(": ");});
        }

        tok.append(quote!{fun(stringify!(#ty));});
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

        tok.append(quote!{let mut #name:ParseTmp<#ty> = ParseTmp::new(stringify!(#name).into());});
        match get_attrs(field).and_then(|x| find_attr_lit("default", x)) {
            Some(x) => {
                match x {
                    &syn::Lit::Str(ref val, _) => {
                        tok.append(quote!{#name.set_default});
                        tok.append(format!("({});", val));
                    }
                    _ => {
                        panic!("default must be a string that will be parsed!");
                    }
                }
            },
            None =>  { },
        }

    }

    tok.append("loop {");
    tok.append(quote!{
        if provider.peek_char() == Some('}') {
            provider.consume(1, fun)?;
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
            provider.consume(stringify!(#name).len(), fun)?;
            provider.consume_char(':', fun)?;
            #name.push_found(<#ty as ConfigAble>::parse_from(provider, fun), provider, fun)?;

            if provider.peek_char() == Some(',') {
                provider.consume(1, fun)?;
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

        tok.append(quote!{<#ty as ConfigAble>::parse_from(provider, fun)?; });
    }
    tok.append(quote!{provider.consume_char(')', fun)?;});
}

fn impl_get_format(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    let mut others = HashSet::new();

    tok.append(quote!{
        #[allow(unused_variables)] /* We need this, since we may not use the set */
        fn get_format<F>(set: &mut ::std::collections::HashSet<String>, fun: &mut F)
            where F: FnMut(&str)
    });
    tok.append("{");

    tok.append(quote!{
        fun(stringify!(#name));
        fun(": ");
    });
    //tok.append(format!("fun(\"{}: \");", name));

    match ast.body {
        /* Handle Enums */
        syn::Body::Enum(ref vars) => {
            let mut first = true;
            for ref var in vars {
                let vname = &var.ident;
                if first {
                    first = false
                } else {
                    tok.append(quote!{fun(" | ");});
                }

                tok.append(quote!{fun(stringify!(#vname));});

                match var.data {
                    syn::VariantData::Unit => {},
                    syn::VariantData::Tuple(ref fields) => {
                        tok.append(quote!{fun("(");});
                        append_fields(fields.iter(), tok, &mut others);
                        tok.append(quote!{fun(")");});
                    },
                    syn::VariantData::Struct(ref fields) => {
                        tok.append(quote!{fun("{");});
                        append_fields(fields.iter(), tok, &mut others);
                        tok.append(quote!{fun("}");});
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
                    tok.append(quote!{fun("(");});
                    append_fields(fields.iter(), tok, &mut others);
                    tok.append(quote!{fun(")");});
                },
                &syn::VariantData::Struct(ref fields) => {
                    tok.append(quote!{fun("{");});
                    append_fields(fields.iter(), tok, &mut others);
                    tok.append(quote!{fun("}");});
                },
            }
        }
    }

    /* Append other types format, so the entire used type tree will be displayed */
    for other in others {
        tok.append(quote!{
            { /* This will be a block, to avoid naming collisions */
                /* Check if we already appended the other type somewhere*/
                let name = stringify!(#other).to_string();
                if !set.contains(&name) {
                    /* If we didn't, insert it into the list of printed types and append it */
                    set.insert(name);

                    fun("\n");
                    <#other as ConfigAble>::get_format(set, fun);
                }
            } /* Close the scoping block */
        });
    }

    tok.append("}"); /* Close print_format */
}

fn impl_get_default(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    tok.append(quote!{fn get_default() -> Result<Self, ()>});
    tok.append("{"); /* Open get_default() function */

    match ast.body {
        syn::Body::Enum(_) => { tok.append(quote!{ return Err(()); }); },
        syn::Body::Struct(ref data) => {
            match data {
                &syn::VariantData::Unit => {
                },
                &syn::VariantData::Tuple(ref fields) => {
                    tok.append("return Ok(");
                    tok.append(quote!{#name});
                    tok.append("(");

                    for (i, ref field) in fields.iter().enumerate() {
                        if i > 0 {
                            tok.append(",");
                        }
                        let ty = &field.ty;

                        tok.append(quote!{<#ty as ConfigAble>::get_default()?});
                    }

                    tok.append("));");
                },
                &syn::VariantData::Struct(ref fields) => {
                    tok.append("return Ok(");
                    tok.append(quote!{#name});
                    tok.append("{");

                    for (i, ref field) in fields.iter().enumerate() {
                        if i > 0 {
                            tok.append(",");
                        }
                        let ty = &field.ty;

                        let name = match field.ident {
                            Some(ref x) => x,
                            None => panic!("Encountered unnamed field while trying to derive named field parsing")
                        };

                        tok.append(quote!{ #name: <#ty as ConfigAble>::get_default()?});
                    }

                    tok.append("});");
                },
            }
        },
    }

    tok.append("}"); /* close get_default() function */
}

fn impl_parse_from(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    tok.append(quote!{#[allow(unused_variables, unreachable_code, unused_assignments)]
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
                                provider.consume(stringify!(#vname).len(), fun)?;
                                return Ok(#name::#vname);
                            }
                        });
                    },
                    syn::VariantData::Tuple(ref fields) => {
                        tok.append(quote!{ if nxt.starts_with(stringify!(#vname))});
                        tok.append("{");
                        tok.append(quote!{ provider.consume(stringify!(#vname).len(), fun)?;});
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
                        tok.append(quote!{ provider.consume(stringify!(#vname).len(), fun)?;});
                        impl_parse_named(fields.iter(), tok);

                        impl_parse_named(fields.iter(), tok);

                        let mut ret_expr = quote::Tokens::new();
                        ret_expr.append("return Ok(");
                        ret_expr.append(quote!{#name::#vname});
                        ret_expr.append("{");

                        for (i, ref field) in fields.iter().enumerate() {
                            if i > 0 {
                                ret_expr.append(",");
                            }

                            let name = match field.ident {
                                Some(ref x) => x,
                                None => panic!("Encountered unnamed field while trying to derive named field parsing")
                            };

                            tok.append(format!("let {}_r = {}.get_value(fun);", name, name));


                            ret_expr.append(format!("{}: {}_r?", name, name));
                        }

                        ret_expr.append("});");
                        ret_expr.append("}");

                        tok.append(ret_expr);
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
                    let mut ret_expr = quote::Tokens::new();
                    ret_expr.append("return Ok(");
                    ret_expr.append(quote!{#name});
                    ret_expr.append("{");

                    for (i, ref field) in fields.iter().enumerate() {
                        if i > 0 {
                            ret_expr.append(",");
                        }

                        let name = match field.ident {
                            Some(ref x) => x,
                            None => panic!("Encountered unnamed field while trying to derive named field parsing")
                        };

                        tok.append(format!("let {}_r = {}.get_value(fun);", name, name));


                        ret_expr.append(format!("{}: {}_r?", name, name));
                    }

                    ret_expr.append("});");

                    tok.append(ret_expr);
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
    impl_get_default(ast, &mut start);

    start.append("}"); /* Close impl */

    return start;
}

#[proc_macro_derive(ConfigAble, attributes(ConfigAttrs))]
pub fn print_format(input: TokenStream) -> TokenStream {
    let s = input.to_string();

    let ast = syn::parse_derive_input(&s).unwrap();

    let gen = impl_derive_config_able(&ast);

    return gen.parse().unwrap();
}

#[cfg(test)]
mod test {

}
