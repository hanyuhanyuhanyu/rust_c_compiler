use super::node::{
    Add, AddSub, Assign, Compare, Equality, Equals, Expr, Lvar, Mul, MulDiv, Primary, PrimaryNode,
    Program, Relational, Stmt, Unary,
};
type GenResult = Result<Vec<String>, Vec<String>>;
fn concat(l: GenResult, r: GenResult) -> GenResult {
    if l.is_err() {
        return l;
    }
    if r.is_err() {
        return r;
    }
    let ll = l.unwrap();
    let rl = r.unwrap();
    Ok([ll, rl].concat())
}
fn concat_multi(results: &[GenResult]) -> GenResult {
    let mut base = Ok(Vec::new());
    for r in results {
        base = concat(base, r.clone());
        if base.is_err() {
            break;
        }
    }
    return base;
}
const NOT_LVAR_ERR: &str = "left value cannot assingable";
fn primary(m: &Primary) -> GenResult {
    match &m.node {
        PrimaryNode::Num(n) => Ok(vec![format!("push {}", n)]),
        PrimaryNode::Expr(e) => expr(&e),
        PrimaryNode::Lv(Lvar::Id(i)) => Ok(vec![
            "mov rax, rbp".into(),
            format!("sub rax, {}", i.offset),
            "push [rax]".into(),
        ]),
    }
}
fn unary(u: &Unary) -> GenResult {
    match u.prim.ope {
        None | Some(AddSub::Plus) => {
            return primary(&u.prim);
        }
        _ => {
            let prim = primary(&u.prim);
            if prim.is_err() {
                return prim;
            }
            let mut lines = [vec![format!("push {}", 0)], prim.unwrap()].concat();
            lines.push("pop rdi".into());
            lines.push("pop rax".into());
            lines.push("sub rax, rdi".into());
            lines.push("push rax".into());
            return Ok(lines);
        }
    }
}
fn mul(m: &Mul) -> GenResult {
    let first = unary(&m.first);
    if first.is_err() {
        return first;
    }
    if m.unarys.len() == 0 {
        return first;
    }
    let mut lines = first.unwrap();
    for u in m.unarys.iter() {
        if u.ope.is_none() {
            return Err(vec!["operator expected".into()]);
        }
        let second = unary(u);
        if second.is_err() {
            return second;
        }
        lines.extend(second.unwrap());
        lines.push("pop rdi".into());
        lines.push("pop rax".into());
        match u.ope.as_ref().unwrap() {
            MulDiv::Multi => {
                lines.push("imul rax,rdi".into());
            }
            MulDiv::Divide => {
                lines.push("cqo".into());
                lines.push("idiv rax,rdi".into());
            }
        }
        lines.push("push rax".into());
    }
    return Ok(lines);
}
fn add(a: &Add) -> GenResult {
    let first = mul(&a.first);
    if first.is_err() {
        return first;
    }
    if a.muls.len() == 0 {
        return first;
    }
    let mut lines = first.unwrap();
    for m in a.muls.iter() {
        if m.ope.is_none() {
            return Err(vec!["operator expected".into()]);
        }
        let second = mul(m);
        if second.is_err() {
            return second;
        }
        lines.extend(second.unwrap());
        lines.push("pop rdi".into());
        lines.push("pop rax".into());
        match m.ope.as_ref().unwrap() {
            AddSub::Plus => {
                lines.push("add rax, rdi".into());
            }
            AddSub::Minus => {
                lines.push("sub rax, rdi".into());
            }
        }
        lines.push("push rax".into());
    }
    return Ok(lines);
}
fn relational(rel: &Relational) -> GenResult {
    let first = add(&rel.first);
    if first.is_err() {
        return first;
    }
    if rel.adds.len() == 0 {
        return first;
    }
    let mut lines = first.unwrap();
    for a in rel.adds.iter() {
        if a.ope.is_none() {
            return Err(vec!["operator expected".into()]);
        }
        let second = add(a);
        if second.is_err() {
            return second;
        }
        lines.extend(second.unwrap());
        lines.push("pop rdi".into());
        lines.push("pop rax".into());
        match a.ope.as_ref().unwrap() {
            Compare::Lt => {
                lines.push("cmp rax, rdi".into());
                lines.push("setl al".into());
            }
            Compare::Lte => {
                lines.push("cmp rax, rdi".into());
                lines.push("setle al".into());
            }
            Compare::Gt => {
                lines.push("cmp rdi, rax".into());
                lines.push("setl al".into());
            }
            Compare::Gte => {
                lines.push("cmp rdi, rax".into());
                lines.push("setle al".into());
            }
        }
        lines.push("movzb rax, al".into());
        lines.push("push rax".into());
    }
    return Ok(lines);
}
fn equality(eq: &Equality) -> GenResult {
    let first = relational(&eq.first);
    if first.is_err() {
        return first;
    }
    if eq.relationals.len() == 0 {
        return first;
    }
    let mut lines = first.unwrap();
    for rel in eq.relationals.iter() {
        if rel.ope.is_none() {
            return Err(vec!["operator expected".into()]);
        }
        let second = relational(rel);
        if second.is_err() {
            return second;
        }
        let ope = rel.ope.as_ref().unwrap();
        lines.append(second.unwrap().as_mut());
        lines.push("pop rdi".into());
        lines.push("pop rax".into());
        lines.push("cmp rax, rdi".into());
        match ope {
            Equals::Equal => lines.push("sete al".into()),
            Equals::NotEqual => lines.push("setne al".into()),
        }
        lines.push("movzb rax, al".into());
        lines.push("push rax".into());
    }
    return Ok(lines);
}

fn lvar(e: &Lvar) -> GenResult {
    match e {
        Lvar::Id(i) => Ok(vec![
            "mov rax, rbp".into(),
            format!("sub rax, {}", i.offset),
            "push rax".into(),
        ]),
    }
}
fn assign(a: &Assign) -> GenResult {
    if a.rvar.is_none() {
        return equality(&a.lvar);
    }
    match a.lvar.lvar() {
        None => Err(vec![NOT_LVAR_ERR.into()]),
        Some(l) => {
            let l = lvar(l);
            let r = assign(a.rvar.as_ref().unwrap());
            let lines = concat(l, r);
            match lines {
                Err(e) => Err(e),
                Ok(ls) => Ok([
                    ls,
                    vec![
                        "pop rdi".into(),
                        "pop rax".into(),
                        "mov [rax], rdi".into(),
                        "push rdi".into(),
                    ],
                ]
                .concat()),
            }
        }
    }
}
fn expr(e: &Expr) -> GenResult {
    assign(&e.assign)
}
fn stmt(s: &Stmt) -> GenResult {
    expr(&s.expr)
}
fn program(p: &Program) -> GenResult {
    let mut lines = Vec::new();
    for s in p.stmt.iter() {
        match stmt(s) {
            Ok(mut l) => {
                lines.append(&mut l);
            }
            Err(e) => {
                return Err(e);
            }
        };
        if !s.expr.ret {
            continue;
        }
        lines.extend(epilogue().unwrap());
    }
    Ok(lines)
}
fn prologue(p: &Program) -> GenResult {
    Ok(vec![
        "push rbp #prlg ->".into(),
        "mov rbp, rsp".into(),
        format!("sub rsp, {} {}", p.required_memory, "#<- prlg"),
    ])
}
fn epilogue() -> GenResult {
    Ok(vec![
        "pop rax #eplg ->".into(),
        "mov rsp, rbp".into(),
        "pop rbp".into(),
        "ret #<- eplg".into(),
    ])
}
pub fn generate(p: &Program) -> GenResult {
    concat_multi(&[prologue(p), program(p), epilogue()])
}
