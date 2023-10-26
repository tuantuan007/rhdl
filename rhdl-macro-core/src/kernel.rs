use quote::{format_ident, quote};
use syn::spanned::Spanned;
type TS = proc_macro2::TokenStream;
type Result<T> = syn::Result<T>;

pub fn hdl_kernel(input: TS) -> Result<TS> {
    let original = input.clone();
    let input = syn::parse::<syn::ItemFn>(input.into())?;
    let name = format_ident!("{}_hdl_kernel", &input.sig.ident);
    let block = hdl_block_inner(&input.block)?;
    Ok(quote! {
        #original

        fn #name() -> Box<rhdl_core::ast::Block> {
            #block
        }
    })
}

fn hdl_block(block: &syn::Block) -> Result<TS> {
    let block = hdl_block_inner(block)?;
    Ok(quote! {
        rhdl_core::ast_builder::block_expr(#block)
    })
}

fn hdl_block_inner(block: &syn::Block) -> Result<TS> {
    let stmts = block.stmts.iter().map(stmt).collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        rhdl_core::ast_builder::block(vec![#(#stmts),*],)
    })
}

fn stmt(statement: &syn::Stmt) -> Result<TS> {
    match statement {
        syn::Stmt::Local(local) => stmt_local(local),
        syn::Stmt::Expr(expr, semi) => {
            let expr = hdl_expr(expr)?;
            if semi.is_some() {
                Ok(quote! {
                    rhdl_core::ast_builder::semi_stmt(#expr)
                })
            } else {
                Ok(quote! {
                    rhdl_core::ast_builder::expr_stmt(#expr)
                })
            }
        }
        _ => Err(syn::Error::new(
            statement.span(),
            "Unsupported statement type",
        )),
    }
}

fn stmt_local(local: &syn::Local) -> Result<TS> {
    let pattern = hdl_pat(&local.pat)?;
    let local_init = local
        .init
        .as_ref()
        .map(|x| hdl_expr(&x.expr))
        .transpose()?
        .map(|x| quote!(Some(#x)))
        .unwrap_or(quote! {None});
    Ok(quote! {
            rhdl_core::ast_builder::local_stmt(#pattern, #local_init)
    })
}

fn hdl_pat(pat: &syn::Pat) -> Result<TS> {
    match pat {
        syn::Pat::Ident(ident) => {
            let name = &ident.ident;
            let mutability = ident.mutability.is_some();
            if ident.by_ref.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "Unsupported reference pattern",
                ));
            }
            Ok(quote! {
                rhdl_core::ast_builder::ident_pat(stringify!(#name).to_string(),#mutability)
            })
        }
        syn::Pat::TupleStruct(tuple) => {
            let path = hdl_path_inner(&tuple.path)?;
            let elems = tuple
                .elems
                .iter()
                .map(hdl_pat)
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! {
                rhdl_core::ast_builder::tuple_struct_pat(#path, vec![#(#elems),*])
            })
        }
        syn::Pat::Tuple(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(hdl_pat)
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! {
                rhdl_core::ast_builder::tuple_pat(vec![#(#elems),*])
            })
        }
        syn::Pat::Path(path) => {
            let path = hdl_path_inner(&path.path)?;
            Ok(quote! {
                rhdl_core::ast_builder::path_pat(#path)
            })
        }
        syn::Pat::Struct(structure) => {
            let path = hdl_path_inner(&structure.path)?;
            let fields = structure
                .fields
                .iter()
                .map(hdl_field_pat)
                .collect::<Result<Vec<_>>>()?;
            if structure.qself.is_some() {
                return Err(syn::Error::new(
                    structure.span(),
                    "Unsupported qualified self in rhdl kernel function",
                ));
            }
            let rest = structure.rest.is_some();
            Ok(quote! {
                rhdl_core::ast_builder::struct_pat(#path, vec![#(#fields),*], #rest)
            })
        }
        syn::Pat::Type(pat) => {
            let ty = &pat.ty;
            let pat = hdl_pat(&pat.pat)?;
            let kind = quote! {<#ty as rhdl_core::Digital>::static_kind()};
            Ok(quote! {
                rhdl_core::ast_builder::type_pat(#pat, #kind)
            })
        }
        syn::Pat::Lit(pat) => {
            let lit = hdl_lit_inner(pat)?;
            Ok(quote! {
                rhdl_core::ast_builder::lit_pat(#lit)
            })
        }
        syn::Pat::Wild(_) => Ok(quote! {
            rhdl_core::ast_builder::wild_pat()
        }),
        _ => Err(syn::Error::new(pat.span(), "Unsupported pattern type")),
    }
}

fn hdl_field_pat(expr: &syn::FieldPat) -> Result<TS> {
    let member = hdl_member(&expr.member)?;
    let pat = hdl_pat(&expr.pat)?;
    Ok(quote! {
        rhdl_core::ast_builder::field_pat(#member, #pat)
    })
}

fn hdl_expr(expr: &syn::Expr) -> Result<TS> {
    match expr {
        syn::Expr::Lit(expr) => hdl_lit(expr),
        syn::Expr::Binary(expr) => hdl_binary(expr),
        syn::Expr::Unary(expr) => hdl_unary(expr),
        syn::Expr::Group(expr) => hdl_group(expr),
        syn::Expr::Paren(expr) => hdl_paren(expr),
        syn::Expr::Assign(expr) => hdl_assign(expr),
        syn::Expr::Path(expr) => hdl_path(&expr.path),
        syn::Expr::Struct(expr) => hdl_struct(expr),
        syn::Expr::Block(expr) => hdl_block(&expr.block),
        syn::Expr::Field(expr) => hdl_field_expression(expr),
        syn::Expr::If(expr) => hdl_if(expr),
        syn::Expr::Let(expr) => hdl_let(expr),
        syn::Expr::Match(expr) => hdl_match(expr),
        syn::Expr::Range(expr) => hdl_range(expr),
        syn::Expr::Try(expr) => hdl_try(expr),
        syn::Expr::Return(expr) => hdl_return(expr),
        syn::Expr::Tuple(expr) => hdl_tuple(expr),
        syn::Expr::Repeat(expr) => hdl_repeat(expr),
        syn::Expr::ForLoop(expr) => hdl_for_loop(expr),
        syn::Expr::While(expr) => hdl_while_loop(expr),
        syn::Expr::Call(expr) => hdl_call(expr),
        syn::Expr::Array(expr) => hdl_array(expr),
        syn::Expr::Index(expr) => hdl_index(expr),
        syn::Expr::MethodCall(expr) => hdl_method_call(expr),
        _ => Err(syn::Error::new(
            expr.span(),
            format!(
                "Unsupported expression type {} in an rhdl kernel function",
                quote!(#expr)
            ),
        )),
    }
}

fn hdl_method_call(expr: &syn::ExprMethodCall) -> Result<TS> {
    let receiver = hdl_expr(&expr.receiver)?;
    let args = expr.args.iter().map(hdl_expr).collect::<Result<Vec<_>>>()?;
    let method = &expr.method;
    Ok(quote! {
        rhdl_core::ast_builder::method_expr(#receiver, vec![#(#args),*], stringify!(#method).to_string())
    })
}

fn hdl_index(expr: &syn::ExprIndex) -> Result<TS> {
    let index = hdl_expr(&expr.index)?;
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::index_expr(#expr, #index)
    })
}

fn hdl_array(expr: &syn::ExprArray) -> Result<TS> {
    let elems = expr
        .elems
        .iter()
        .map(hdl_expr)
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        rhdl_core::ast_builder::array_expr(vec![#(#elems),*])
    })
}

fn hdl_call(expr: &syn::ExprCall) -> Result<TS> {
    let syn::Expr::Path(func_path) = expr.func.as_ref() else {
        return Err(syn::Error::new(
            expr.func.span(),
            "Unsupported function call in rhdl kernel function (only paths allowed here)",
        ));
    };
    let path = hdl_path_inner(&func_path.path)?;
    let args = expr.args.iter().map(hdl_expr).collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        rhdl_core::ast_builder::call_expr(#path, vec![#(#args),*])
    })
}

fn hdl_for_loop(expr: &syn::ExprForLoop) -> Result<TS> {
    let pat = hdl_pat(&expr.pat)?;
    let body = hdl_block_inner(&expr.body)?;
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::for_expr(#pat, #expr, #body)
    })
}

fn hdl_while_loop(expr: &syn::ExprWhile) -> Result<TS> {
    // In version 2.0...
    Err(syn::Error::new(
        expr.span(),
        "Unsupported while loop in rhdl kernel function",
    ))
}

fn hdl_repeat(expr: &syn::ExprRepeat) -> Result<TS> {
    let len = hdl_expr(&expr.len)?;
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::repeat_expr(#expr, #len)
    })
}

fn hdl_tuple(expr: &syn::ExprTuple) -> Result<TS> {
    let elems = expr
        .elems
        .iter()
        .map(hdl_expr)
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        rhdl_core::ast_builder::tuple_expr(vec![#(#elems),*])
    })
}

fn hdl_group(expr: &syn::ExprGroup) -> Result<TS> {
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::group_expr(#expr)
    })
}

fn hdl_paren(expr: &syn::ExprParen) -> Result<TS> {
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::paren_expr(#expr)
    })
}

fn hdl_return(expr: &syn::ExprReturn) -> Result<TS> {
    let expr = expr
        .expr
        .as_ref()
        .map(|x| hdl_expr(x))
        .transpose()?
        .map(|x| quote! {Some(#x)})
        .unwrap_or_else(|| quote! {None});
    Ok(quote! {
        rhdl_core::ast_builder::return_expr(#expr)
    })
}

fn hdl_try(expr: &syn::ExprTry) -> Result<TS> {
    Err(syn::Error::new(
        expr.span(),
        "Unsupported try expression in rhdl kernel function",
    ))
}

fn hdl_range(expr: &syn::ExprRange) -> Result<TS> {
    let start = expr
        .start
        .as_ref()
        .map(|x| hdl_expr(x))
        .transpose()?
        .map(|x| quote! {Some(#x)})
        .unwrap_or_else(|| quote! {None});
    let end = expr
        .end
        .as_ref()
        .map(|x| hdl_expr(x))
        .transpose()?
        .map(|x| quote! {Some(#x)})
        .unwrap_or_else(|| quote! {None});
    let limits = match expr.limits {
        syn::RangeLimits::HalfOpen(_) => quote!(rhdl_core::ast_builder::range_limits_half_open()),
        syn::RangeLimits::Closed(_) => quote!(rhdl_core::ast_builder::range_limits_closed()),
    };
    Ok(quote! {
        rhdl_core::ast_builder::range_expr(#start, #limits, #end)
    })
}

fn hdl_match(expr: &syn::ExprMatch) -> Result<TS> {
    let arms = expr.arms.iter().map(hdl_arm).collect::<Result<Vec<_>>>()?;
    let expr = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::match_expr(#expr, vec![#(#arms),*])
    })
}

fn literal_or_ranges(pat: &syn::Pat) -> bool {
    match pat {
        syn::Pat::Lit(_) => true,
        syn::Pat::Range(_) => true,
        syn::Pat::Paren(pat) => literal_or_ranges(&pat.pat),
        syn::Pat::TupleStruct(tuple) => tuple.elems.iter().any(literal_or_ranges),
        syn::Pat::Struct(structure) => structure.fields.iter().any(|x| literal_or_ranges(&x.pat)),
        syn::Pat::Or(pat) => pat.cases.iter().any(literal_or_ranges),
        _ => false,
    }
}

fn ident_or_wildcard(pat: &syn::Pat) -> bool {
    matches!(pat, syn::Pat::Ident(_) | syn::Pat::Wild(_))
}

fn hdl_pat_arm(pat: &syn::Pat) -> Result<TS> {
    // Here (or a level above) - we need to check for the
    // all literal + wildcard case or the enum case.
    // We should also restrict the enum case so that all
    // paths are the same.  And that path should correspond
    // to a Digital type.

    // If the top level pattern is a TupleStruct, or a Struct,
    // then we need to ensure there are no literal or
    // range patterns in the fields.

    match pat {
        syn::Pat::TupleStruct(tuple) => {
            if !tuple.elems.iter().all(ident_or_wildcard) {
                return Err(syn::Error::new(
                    tuple.span(),
                    "Unsupported tuple struct pattern - rhdl only supports simple patterns like Foo(a,b,_)",
                ));
            }
        }
        syn::Pat::Struct(structure) => {
            if !structure.fields.iter().all(|x| ident_or_wildcard(&x.pat)) {
                return Err(syn::Error::new(
                    structure.span(),
                    "Unsupported literal or range in struct pattern",
                ));
            }
        }
        syn::Pat::Path(path) => {
            if path.qself.is_some() {
                return Err(syn::Error::new(
                    path.span(),
                    "Unsupported qualified self in rhdl kernel function",
                ));
            }
        }
        syn::Pat::Lit(_) | syn::Pat::Wild(_) => {}
        _ => {
            return Err(syn::Error::new(
                pat.span(),
                "Unsupported match pattern in rhdl kernel function",
            ))
        }
    }
    hdl_pat(pat)
}

fn hdl_arm(arm: &syn::Arm) -> Result<TS> {
    let pat = hdl_pat_arm(&arm.pat)?;
    let guard = arm
        .guard
        .as_ref()
        .map(|(_if, x)| hdl_expr(x))
        .transpose()?
        .map(|x| quote! {Some(#x)})
        .unwrap_or(quote! {None});
    let body = hdl_expr(&arm.body)?;
    Ok(quote! {
        rhdl_core::ast_builder::arm(#pat, #guard, #body)
    })
}

fn hdl_let(expr: &syn::ExprLet) -> Result<TS> {
    let pattern = hdl_pat(&expr.pat)?;
    let value = hdl_expr(&expr.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::let_expr(#pattern, #value)
    })
}

fn hdl_if(expr: &syn::ExprIf) -> Result<TS> {
    let cond = hdl_expr(&expr.cond)?;
    let then = hdl_block_inner(&expr.then_branch)?;
    let else_ = expr
        .else_branch
        .as_ref()
        .map(|x| hdl_expr(&x.1))
        .transpose()?
        .map(|x| quote! {Some(#x)})
        .unwrap_or(quote! {None});
    Ok(quote! {
        rhdl_core::ast_builder::if_expr(#cond, #then, #else_)
    })
}

fn hdl_struct(structure: &syn::ExprStruct) -> Result<TS> {
    let path = hdl_path_inner(&structure.path)?;
    let fields = structure
        .fields
        .iter()
        .map(hdl_field_value)
        .collect::<Result<Vec<_>>>()?;
    if structure.qself.is_some() {
        return Err(syn::Error::new(
            structure.span(),
            "Unsupported qualified self in rhdl kernel function",
        ));
    }
    let rest = structure
        .rest
        .as_ref()
        .map(|x| hdl_expr(x))
        .transpose()?
        .unwrap_or(quote! {None});
    Ok(quote! {
        rhdl_core::ast_builder::struct_expr(#path,vec![#(#fields),*],#rest),
    })
}

fn hdl_path(path: &syn::Path) -> Result<TS> {
    let inner = hdl_path_inner(path)?;
    Ok(quote! {
        rhdl_core::ast_builder::path_expr(#inner)
    })
}

fn hdl_path_inner(path: &syn::Path) -> Result<TS> {
    let segments = path
        .segments
        .iter()
        .map(hdl_path_segment)
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        rhdl_core::ast_builder::path(vec![#(#segments),*],)
    })
}

fn hdl_path_segment(segment: &syn::PathSegment) -> Result<TS> {
    let ident = &segment.ident;
    Ok(quote! {
        rhdl_core::ast_builder::path_segment(stringify!(#ident).to_string())
    })
}

fn hdl_assign(assign: &syn::ExprAssign) -> Result<TS> {
    let left = hdl_expr(&assign.left)?;
    let right = hdl_expr(&assign.right)?;
    Ok(quote! {
        rhdl_core::ast_builder::assign_expr(#left, #right)
    })
}

fn hdl_field_expression(field: &syn::ExprField) -> Result<TS> {
    let expr = hdl_expr(&field.base)?;
    let member = hdl_member(&field.member)?;
    Ok(quote! {
        rhdl_core::ast_builder::field_expr(#expr, #member)
    })
}

fn hdl_field_value(field: &syn::FieldValue) -> Result<TS> {
    let member = hdl_member(&field.member)?;
    let value = hdl_expr(&field.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::field_value(#member, #value)
    })
}

fn hdl_member(member: &syn::Member) -> Result<TS> {
    Ok(match member {
        syn::Member::Named(ident) => quote! {
            rhdl_core::ast_builder::member_named(stringify!(#ident).to_string())
        },
        syn::Member::Unnamed(index) => {
            let index = index.index;
            quote! {
                rhdl_core::ast_builder::member_unnamed(#index)
            }
        }
    })
}

fn hdl_unary(unary: &syn::ExprUnary) -> Result<TS> {
    let op = match unary.op {
        syn::UnOp::Neg(_) => quote!(rhdl_core::ast_builder::UnOp::Neg),
        syn::UnOp::Not(_) => quote!(rhdl_core::ast_builder::UnOp::Not),
        _ => {
            return Err(syn::Error::new(
                unary.span(),
                "Unsupported unary operator in rhdl kernel function",
            ))
        }
    };
    let expr = hdl_expr(&unary.expr)?;
    Ok(quote! {
        rhdl_core::ast_builder::unary_expr(#op, #expr)
    })
}

fn hdl_binary(binary: &syn::ExprBinary) -> Result<TS> {
    let op = match binary.op {
        syn::BinOp::Add(_) => quote!(rhdl_core::ast_builder::BinOp::Add),
        syn::BinOp::Sub(_) => quote!(rhdl_core::ast_builder::BinOp::Sub),
        syn::BinOp::Mul(_) => quote!(rhdl_core::ast_builder::BinOp::Mul),
        syn::BinOp::And(_) => quote!(rhdl_core::ast_builder::BinOp::And),
        syn::BinOp::Or(_) => quote!(rhdl_core::ast_builder::BinOp::Or),
        syn::BinOp::BitXor(_) => quote!(rhdl_core::ast_builder::BinOp::BitXor),
        syn::BinOp::BitAnd(_) => quote!(rhdl_core::ast_builder::BinOp::BitAnd),
        syn::BinOp::BitOr(_) => quote!(rhdl_core::ast_builder::BinOp::BitOr),
        syn::BinOp::Shl(_) => quote!(rhdl_core::ast_builder::BinOp::Shl),
        syn::BinOp::Shr(_) => quote!(rhdl_core::ast_builder::BinOp::Shr),
        syn::BinOp::Eq(_) => quote!(rhdl_core::ast_builder::BinOp::Eq),
        syn::BinOp::Lt(_) => quote!(rhdl_core::ast_builder::BinOp::Lt),
        syn::BinOp::Le(_) => quote!(rhdl_core::ast_builder::BinOp::Le),
        syn::BinOp::Ne(_) => quote!(rhdl_core::ast_builder::BinOp::Ne),
        syn::BinOp::Ge(_) => quote!(rhdl_core::ast_builder::BinOp::Ge),
        syn::BinOp::Gt(_) => quote!(rhdl_core::ast_builder::BinOp::Gt),
        syn::BinOp::AddAssign(_) => quote!(rhdl_core::ast_builder::BinOp::AddAssign),
        syn::BinOp::SubAssign(_) => quote!(rhdl_core::ast_builder::BinOp::SubAssign),
        syn::BinOp::MulAssign(_) => quote!(rhdl_core::ast_builder::BinOp::MulAssign),
        syn::BinOp::BitXorAssign(_) => quote!(rhdl_core::ast_builder::BinOp::BitXorAssign),
        syn::BinOp::BitAndAssign(_) => quote!(rhdl_core::ast_builder::BinOp::BitAndAssign),
        syn::BinOp::BitOrAssign(_) => quote!(rhdl_core::ast_builder::BinOp::BitOrAssign),
        syn::BinOp::ShlAssign(_) => quote!(rhdl_core::ast_builder::BinOp::ShlAssign),
        syn::BinOp::ShrAssign(_) => quote!(rhdl_core::ast_builder::BinOp::ShrAssign),
        _ => {
            return Err(syn::Error::new(
                binary.span(),
                "Unsupported binary operator in rhdl kernel function",
            ))
        }
    };
    let left = hdl_expr(&binary.left)?;
    let right = hdl_expr(&binary.right)?;
    Ok(quote! {
        rhdl_core::ast_builder::binary_expr(#op, #left, #right)
    })
}

fn hdl_lit(lit: &syn::ExprLit) -> Result<TS> {
    let inner = hdl_lit_inner(lit)?;
    Ok(quote! {
        rhdl_core::ast_builder::lit_expr(#inner)
    })
}

fn hdl_lit_inner(lit: &syn::ExprLit) -> Result<TS> {
    let lit = &lit.lit;
    match lit {
        syn::Lit::Int(int) => {
            let value = int.token();
            Ok(quote! {
                    rhdl_core::ast_builder::expr_lit_int(stringify!(#value).to_string())
            })
        }
        syn::Lit::Bool(boolean) => {
            let value = boolean.value;
            Ok(quote! {
                    rhdl_core::ast_builder::expr_lit_bool(#value)
            })
        }
        _ => Err(syn::Error::new(
            lit.span(),
            "Unsupported literal type in rhdl kernel function",
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_block() {
        let test_code = quote! {
            {
                let a = 1;
                let b = 2;
                let q = 0x1234_u32;
                let c = a + b;
                let mut d = 3;
                let g = Foo {r: 1, g: 120, b: 33};
                let h = g.r;
                c
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = hdl_block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }
    #[test]
    fn test_precedence_parser() {
        let test_code = quote! {
            {
                1 + 3 * 9
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = hdl_block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_struct_expression_let() {
        let test_code = quote! {
            let d = Foo {a: 1, b: 2};
        };
        let local = syn::parse2::<syn::Stmt>(test_code).unwrap();
        let result = stmt(&local).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_if_expression() {
        let test_code = quote! {
            if d > 0 {
                d = d - 1;
            }
        };
        let if_expr = syn::parse2::<syn::Stmt>(test_code).unwrap();
        let result = stmt(&if_expr).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_syn_match() {
        let test_code = quote! {
            match l {
                State::Init => {}
                State::Run(a) => {}
                State::Boom => {}
                State::NotOk(3) => {}
            }
        };
        let match_expr = syn::parse2::<syn::Stmt>(test_code).unwrap();
        println!("{:#?}", match_expr);
    }

    #[test]
    fn test_match_expression() {
        let test_code = quote! {
            match l {
                State::Init => {}
                State::Run(a) => {}
                State::Boom => {}
            }
        };
        let match_expr = syn::parse2::<syn::Stmt>(test_code).unwrap();
        let result = stmt(&match_expr).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        //        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_self_update() {
        let test_code = quote! {
            (a,b,c) = 3;
        };
        let assign = syn::parse2::<syn::Stmt>(test_code).unwrap();
        let result = stmt(&assign).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        //        let result = result.replace("rhdl_core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }
}