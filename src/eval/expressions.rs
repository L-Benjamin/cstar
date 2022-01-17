use super::*;

// Evaluates an expression.
pub fn eval_expr(ctx: &Context, expr: &ast::Expr) -> Result<Var> {
    match expr {
        ast::Expr::Ternary(ternary) => eval_ternary(ctx, ternary),
        ast::Expr::Assign(assign) => eval_assign(ctx, assign),
        ast::Expr::Atom(atom) => eval_atom(ctx, atom),
        ast::Expr::LValue(lvalue) => eval_lvalue(ctx, lvalue),
        ast::Expr::ListInit(list_init) => eval_list_init(ctx, list_init),
        ast::Expr::StructInit(struct_init) => eval_struct_init(ctx, struct_init),
        ast::Expr::Call(call) => eval_call(ctx, call),
        ast::Expr::BinExpr(bin_expr) => eval_bin_expr(ctx, bin_expr),
        ast::Expr::UnExpr(un_expr) => eval_un_expr(ctx, un_expr),
    }
}

// Evaluates a ternary expression.
pub fn eval_ternary(ctx: &Context, ternary: &ast::Ternary) -> Result<Var> {
    match eval_expr(ctx, &ternary.cond)? {
        Var::Bool(true) => eval_expr(ctx, &ternary.branch1),
        Var::Bool(false) => eval_expr(ctx, &ternary.branch2),
        _ => return Err(anyhow!("A condition expression evaluated to a non-boolean value in an if statement.")),
    }
}

// Evaluates an atom.
pub fn eval_atom(_: &Context, atom: &ast::Atom) -> Result<Var> {
    Ok(match atom {
        ast::Atom::Void => Var::Void,
        ast::Atom::Bool(b) => Var::Bool(*b),
        ast::Atom::Int(i) => Var::Int(*i),
        ast::Atom::Float(x) => Var::Float(*x),
        ast::Atom::Char(c) => Var::Char(*c),
        ast::Atom::String(s) => Var::String(s.clone()),
    })
}

// Evaluates a list initialization.
pub fn eval_list_init(ctx: &Context, list_init: &ast::ListInit) -> Result<Var> {
    Ok(Var::List(as_ref(list_init.exprs.iter().map(|expr| eval_expr(ctx, expr)).collect::<Result<_>>()?)))
}

// Evaluates a struct initialization.
pub fn eval_struct_init(ctx: &Context, struct_init: &ast::StructInit) -> Result<Var> {
    match ctx.get_def(struct_init.name)? {
        Def::Component(def) | Def::Resource(def) | Def::Struct(def) => {
            let mut map = Map::with_capacity(def.fields.len());

            for (name, expr) in struct_init.fields.iter() {
                if !def.fields.contains_key(name) {
                    return Err(anyhow!("{} is not a field of {}.", name, struct_init.name));
                }

                if map.insert(name, eval_expr(ctx, expr)?).is_some() {
                    return Err(anyhow!("{} is already initialized.", name));
                }
            }

            if def.fields.len() != map.len() {
                return Err(anyhow!("{} has {} fields, but {} fields were given.", struct_init.name, def.fields.len(), map.len()));
            }

            Ok(Var::Struct(as_ref(Struct {
                name: struct_init.name, 
                map
            })))
        },
        _ => return Err(anyhow!("{} is not a struct type.", struct_init.name)),
    }
}