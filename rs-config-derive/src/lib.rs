extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use std::collections::HashSet;


fn get_meta_attrs(attrs: &Vec<syn::Attribute>) -> Option<&Vec<syn::NestedMetaItem>> {
    for attr in attrs {
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

fn get_attrs(field: &syn::Field) -> Option<&Vec<syn::NestedMetaItem>> {
    return get_meta_attrs(&field.attrs);
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

        tok.append(quote!{let mut #name:rs_config::ParseTmp<#ty> = rs_config::ParseTmp::new(stringify!(#name).into());});
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
                return Err(rs_config::ParseError::Final);
            }
        };
    });

    for ref field in fields {
        let name = match field.ident {
            Some(ref x) => x,
            None => panic!("Encountered unnamed field while trying to derive named field parsing")
        };
        let ty = &field.ty;

        tok.append(quote!{
            if nxt.starts_with(stringify!(#name)) {
                provider.consume(stringify!(#name).len(), fun)?;
                provider.consume_char(':', fun)?;
                #name.push_found(<#ty as ConfigAble>::parse_from(provider, fun), provider, fun)?;

                if provider.peek_char() == Some(',') {
                    provider.consume(1, fun)?;
                }
                continue;
            }
        });
    }

    tok.append(quote!{
        provider.print_error(0, fun);
        fun(format!("Found invalid field name !{}!", nxt));
        return Err(rs_config::ParseError::Final);
    });

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

    if let Some(x) =  get_meta_attrs(&ast.attrs).and_then(|x| find_attr_lit("default", x)) {
        match x {
            &syn::Lit::Str(ref val, _) => {
                tok.append("Ok(");
                tok.append(val);
                tok.append(")");
            }
            _ => {
                panic!("merge must be a string that's a function name!");
            }
        }

        tok.append("}"); /* close get_default() function */
        return;
    }

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

fn impl_merge(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    tok.append(quote!{
        #[allow(unused_variables, unreachable_code, unreachable_patterns)]
        fn merge(&mut self, rhs: Self) -> Result<(), ()>
    });
    tok.append("{"); /* open merge function */

    match get_meta_attrs(&ast.attrs).and_then(|x| find_attr_lit("merge", x)) {
        Some(x) => {
            match x {
                &syn::Lit::Str(ref val, _) => {
                    tok.append("return self.");
                    tok.append(val);
                    tok.append("(rhs); }");
                    return;
                }
                _ => {
                    panic!("merge must be a string that's a function name!");
                }
            }
        },
        None =>  { },
    }

    match ast.body {
        /* Handle Enums */
        syn::Body::Enum(ref vars) => {
            tok.append("match self {"); /* Open self matching */
            for ref var in vars {
                let vname = &var.ident;
                match var.data {
                    syn::VariantData::Unit => {
                        tok.append(quote!{
                            &mut #name::#vname => {
                                match rhs {
                                    #name::#vname => {
                                        return Ok(());
                                    },
                                    _ => {
                                        return Err(());
                                    },
                                }
                            },
                        });
                    },
                    syn::VariantData::Tuple(ref fields) => {
                        let mut partial = quote::Tokens::new();
                        let mut lhs_fields = quote::Tokens::new();
                        let mut rhs_fields = quote::Tokens::new();
                        let mut merger = quote::Tokens::new();

                        lhs_fields.append("(");
                        rhs_fields.append("(");

                        for (i, _) in fields.iter().enumerate() {

                            let name = format!("var{}", i);

                            if i > 0 {
                                lhs_fields.append(",");
                                rhs_fields.append(",");
                            }

                            lhs_fields.append(format!("ref mut l_{}", name));
                            rhs_fields.append(format!("r_{}", name));
                            merger.append(format!("l_{}.merge(r_{})?;", name, name));
                        }

                        lhs_fields.append(")");
                        rhs_fields.append(")");

                        partial.append(quote!{&mut #name::#vname});
                        partial.append(lhs_fields);
                        partial.append(" => {"); /* Open lhs match */
                        partial.append("match rhs {");

                        partial.append(quote!{#name::#vname});
                        partial.append(rhs_fields);
                        partial.append("=> {"); /* Open rhs match */

                        partial.append(merger);
                        partial.append("return Ok(());");

                        partial.append("},"); /* Close rhs match */
                        partial.append("_ => {return Err(());},}"); /* Close rhs matching */
                        partial.append("},"); /* close lhs match */

                        tok.append(partial);
                    },
                    syn::VariantData::Struct(ref fields) => {
                        let mut lhs_fields = quote::Tokens::new();
                        let mut rhs_fields = quote::Tokens::new();
                        let mut merger = quote::Tokens::new();

                        lhs_fields.append("{");
                        rhs_fields.append("{");

                        for (i, ref field) in fields.iter().enumerate() {

                            let name = match field.ident {
                                Some(ref x) => x,
                                None => panic!("Encountered unnamed field while trying to derive named field parsing")
                            };

                            if i > 0 {
                                lhs_fields.append(",");
                                rhs_fields.append(",");
                            }

                            lhs_fields.append(format!("{}: ref mut l_{}", name, name));
                            rhs_fields.append(format!("{}: r_{}", name, name));
                            merger.append(format!("l_{}.merge(r_{})?;", name, name));
                        }

                        lhs_fields.append("}");
                        rhs_fields.append("}");

                        tok.append(quote!{&mut #name::#vname});
                        tok.append(lhs_fields);
                        tok.append(" => {"); /* Open lhs match */
                        tok.append("match rhs {");

                        tok.append(quote!{#name::#vname});
                        tok.append(rhs_fields);
                        tok.append("=> {"); /* Open rhs match */

                        tok.append(merger);
                        tok.append("return Ok(());");

                        tok.append("},"); /* Close rhs match */
                        tok.append("_ => {return Err(());},}"); /* Close rhs matching */
                        tok.append("},"); /* close lhs match */
                    },
                }
                /* We don't do Tuple merging (yet), so we ignore everything */
            }
            tok.append("}");/* Close self matching */
        },
        /* Handle structs */
        syn::Body::Struct(ref data) => {

            match data {
                &syn::VariantData::Unit => {
                },
                &syn::VariantData::Tuple(ref fields) => {
                    for (i, _) in fields.iter().enumerate() {
                        tok.append(format!("self.{}.merge(rhs.{})?;", i, i));
                    }
                    tok.append(quote!{return Ok(());});
                },
                &syn::VariantData::Struct(ref fields) => {
                    /* Since a merge error is considered an error, it doesn't matter that this
                     * is impure and sets a few values, even when it fails
                     */
                    for ref field in fields.iter() {
                        let name = match field.ident {
                            Some(ref x) => x,
                            None => panic!("Encountered unnamed field while trying to derive named field parsing")
                        };

                        tok.append(quote!{ self.#name.merge(rhs.#name)?; });
                    }
                    tok.append(quote!{return Ok(());});
                },
            }
        }
    }

    tok.append("return Err(());");
    tok.append("}"); /* close merge function */
}

fn impl_parse_from(ast: &syn::MacroInput, tok: &mut quote::Tokens) {
    let name = &ast.ident;
    tok.append(quote!{#[allow(unused_variables, unreachable_code, unused_assignments)]
        fn parse_from<F>(provider: &mut rs_config::ConfigProvider, fun: &mut F) -> Result<Self, rs_config::ParseError>
           where  F: FnMut(String)
    });
    tok.append("{");
    tok.append(quote!{let nxt = match provider.get_next() {
            Some(x) => x,
            None => {
                fun("Was at end of file.".to_string());
                return Err(rs_config::ParseError::Final);
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
        return Err(rs_config::ParseError::Final);
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
    impl_merge(ast, &mut start);

    start.append("}"); /* Close impl */

//    println!("{:?}", start); /* debug output of entire derived trait */
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
