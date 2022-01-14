use anyhow::Result;

use crate::ast;

use super::*;

// Evaluates an expression.
pub fn eval_expr<'a>(scope: &Scope<'a>, ctx: &'a Context<'a>, expr: &ast::Expr<'a>) -> Result<Var<'a>> {
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

// Evaluates an assignment expression.
pub fn eval_assign<'a>(scope: &Scope<'a>, ctx: &'a Context<'a>, assign: &ast::Assign<'a>) -> Result<Var<'a>> {
    let val = eval_expr(scope, ctx, &assign.expr)?;

    scope.mutate_var(&assign.lvalue.path, |var| {
        // Structs can't be reassigned to.
        if matches!(var, Var::Struct {..}) {
            return Err(anyhow!("Cannot reassign to struct."));
        }

        // In structs, types must be checked.
        if assign.lvalue.path.len() > 1 && val.get_type() != var.get_type() {
            return Err(anyhow!("Type mismatch: cannot assign {} to {}.", val.get_type(), var.get_type()));
        }

        *var = val.clone();
        Ok(())
    })?;

    Ok(val)
}

// Evaluates a ternary expression.
pub fn eval_ternary<'a>(scope: &Scope<'a>, ctx: &'a Context<'a>, ternary: &ast::Ternary<'a>) -> Result<Var<'a>> {
    match eval_expr(scope, ctx, &ternary.cond)? {
        Var::Bool(true) => eval_expr(scope, ctx, &ternary.branch1),
        Var::Bool(false) => eval_expr(scope, ctx, &ternary.branch2),
        _ => return Err(anyhow!("A condition expression evaluated to a non-boolean value in an if statement.")),
    }
}

// Evaluates an atom.
pub fn eval_atom<'a>(scope: &Scope<'a>, ctx: &'a Context<'a>, atom: &ast::Atom) -> Result<Var<'a>> {
    Ok(match atom {
        ast::Atom::Void => Var::Void,
        ast::Atom::Bool(b) => Var::Bool(*b),
        ast::Atom::Int(i) => Var::Int(*i),
        ast::Atom::Float(x) => Var::Float(*x),
        ast::Atom::Char(c) => Var::Char(*c),
        ast::Atom::String(s) => Var::String(s.clone()),
    })
}

// Evaluates a left value.
pub fn eval_lvalue<'a>(scope: &Scope<'a>, ctx: &'a Context<'a>, lvalue: &ast::LValue<'a>) -> Result<Var<'a>> {
    scope.get_var(&lvalue.path, |var| Ok(Var::clone(var)))
}

// Evaluates a struct initialization.
pub fn eval_struct_init<'a>(scope: &Scope<'a>, ctx: &'a Context<'a>, struct_init: &ast::StructInit<'a>) -> Result<Var<'a>> {
    match ctx.get_def(struct_init.name)? {
        Def::Component(def) | Def::Resource(def) | Def::Struct(def) => {
            let mut map = Map::with_capacity(def.fields.len());

            for (name, expr) in struct_init.fields.iter() {
                if !def.fields.contains_key(name) {
                    return Err(anyhow!("{} is not a field of {}.", name, struct_init.name));
                }

                if map.insert(name, eval_expr(scope, ctx, expr)?).is_some() {
                    return Err(anyhow!("{} is already initialized.", name));
                }
            }

            if def.fields.len() != map.len() {
                return Err(anyhow!("{} has {} fields, but {} fields were given.", struct_init.name, def.fields.len(), map.len()));
            }

            Ok(Var::Struct {name: struct_init.name, map})
        },
        _ => return Err(anyhow!("{} is not a struct type.", struct_init.name)),
    }
}