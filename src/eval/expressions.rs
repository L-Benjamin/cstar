use anyhow::Result;

use crate::ast;

use super::*;

/// Evaluates an expression.
pub fn eval_expr<'a>(scope: &'a Scope<'a>, ctx: &Context<'a>, expr: &ast::Expr<'a>) -> Result<Var<'a>> {
    match expr {
        ast::Expr::Ternary(ternary) => eval_ternary(scope, ctx, ternary),
        ast::Expr::Assign(assign) => eval_assign(scope, ctx, assign),
        ast::Expr::Atom(atom) => eval_atom(scope, ctx, atom),
        ast::Expr::LValue(lvalue) => eval_lvalue(scope, ctx, lvalue),
        ast::Expr::StructInit(struct_init) => eval_struct_init(scope, ctx, struct_init),
        ast::Expr::Call(call) => eval_call(scope, ctx, call),
        ast::Expr::BinExpr(bin_expr) => eval_bin_expr(scope, ctx, bin_expr),
        ast::Expr::UnExpr(un_expr) => eval_un_expr(scope, ctx, un_expr),
    }
}

/// Evaluates an assignment expression.
pub fn eval_assign<'a>(scope: &'a Scope<'a>, ctx: &Context<'a>, assign: &ast::Assign<'a>) -> Result<Var<'a>> {
    let val = eval_expr(scope, ctx, &assign.expr)?;

    match &assign.lvalue {
        ast::LValue::Ident(ident) => {
            scope.set_var(ident, val.clone());
        },
        ast::LValue::Access(path) => {
            let mut var = scope.get_var(&path[0])?;
            let mut cur = &mut var;

            for ident in &path[1 .. path.len()-1] {
                match cur {
                    Var::Struct(map) => cur = map.get_mut(ident).ok_or_else(|| anyhow::anyhow!("No field named {}.", ident))?,
                    _ => return Err(anyhow!("{} is not a struct, cannot access it's fields.", ident)),
                }
            }

            match cur {
                Var::Struct(map) => {
                    if let Some(var) = map.get_mut(&path[path.len()-1]) {
                        *var = val.clone();
                    } else {
                        return Err(anyhow!("No field named {}.", path[path.len()-1]));
                    }
                },
                _ => return Err(anyhow!("{} is not a struct, cannot access it's fields.", path[0])),
            }

            scope.set_var(&path[0], var);
        },
    }

    Ok(val)
}

pub fn eval_ternary<'a>(scope: &'a Scope<'a>, ctx: &Context<'a>, ternary: &ast::Ternary<'a>) -> Result<Var<'a>> {
    match eval_expr(scope, ctx, &ternary.cond)? {
        Var::Bool(true) => eval_expr(scope, ctx, &ternary.branch1),
        Var::Bool(false) => eval_expr(scope, ctx, &ternary.branch2),
        _ => return Err(anyhow!("A condition expression evaluated to a non-boolean value in an if statement.")),
    }
}

/// Evaluates an atom.
pub fn eval_atom<'a>(scope: &Scope<'a>, ctx: &Context<'a>, atom: &ast::Atom) -> Result<Var<'a>> {
    Ok(match atom {
        ast::Atom::Void => Var::Void,
        ast::Atom::Bool(b) => Var::Bool(*b),
        ast::Atom::Int(i) => Var::Int(*i),
        ast::Atom::Float(x) => Var::Float(*x),
        ast::Atom::Char(c) => Var::Char(*c),
        ast::Atom::String(s) => Var::String(s.clone()),
    })
}

/// Evaluates a left value.
pub fn eval_lvalue<'a>(scope: &'a Scope<'a>, ctx: &Context<'a>, lvalue: &ast::LValue<'a>) -> Result<Var<'a>> {
    match lvalue {
        ast::LValue::Ident(ident) => scope.get_var(ident),
        ast::LValue::Access(path) => {
            let mut var = &scope.get_var(&path[0])?;

            for ident in &path[1..] {
                match var {
                    Var::Struct(map) => var = map.get(ident).ok_or_else(|| anyhow!("No field named {}.", ident))?,
                    _ => return Err(anyhow!("{} is not a struct, cannot access it's fields.", ident)),
                }
            }

            Ok(var.clone())
        },
    }
}

/// Evaluates a struct initialization.
pub fn eval_struct_init<'a>(scope: &'a Scope<'a>, ctx: &Context<'a>, struct_init: &ast::StructInit<'a>) -> Result<Var<'a>> {
    match ctx.get_def(struct_init.name)? {
        Def::Component(blueprint) | Def::Resource(blueprint) => {
            let mut map = Map::with_capacity(blueprint.names.len());

            for (name, expr) in struct_init.fields.iter() {
                if !blueprint.names.contains(name) {
                    return Err(anyhow!("{} is not a field of {}.", name, struct_init.name));
                }

                map.insert(name, eval_expr(scope, ctx, expr)?);
            }

            if blueprint.names.len() != map.len() {
                return Err(anyhow!("{} has {} fields, but {} fields were given.", struct_init.name, blueprint.names.len(), map.len()));
            }

            Ok(Var::Struct(map))
        },
        _ => return Err(anyhow!("{} is not a struct type.", struct_init.name)),
    }
}

pub fn eval_call<'a>(scope: &'a Scope<'a>, ctx: &Context<'a>, call: &ast::Call<'a>) -> Result<Var<'a>> {
    let n = call.args.len();

    match call.builtin {
        ast::BuiltIn::Clone => todo!(),
        ast::BuiltIn::Spawn => todo!(),
        ast::BuiltIn::Delete => todo!(),
        ast::BuiltIn::Print => {
            if n != 1 {
                return Err(anyhow!("{:?} takes exactly one argument.", call.builtin));
            }

            print!("{}", eval_expr(scope, ctx, &call.args[0])?);
        }
    }

    Ok(Var::Void)
}