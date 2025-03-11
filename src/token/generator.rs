use super::node::{
    Add, AddSub, Compare, Equality, Equals, Expr, Mul, MulDiv, Node, Primary, Relational, Unary,
};
struct Generator {
    nodes: Vec<Box<dyn Node>>,
    index: usize,
}
type GenResult = Result<Vec<String>, Vec<String>>;
fn primary(m: &Primary) -> GenResult {
    let num = m.num.as_ref();
    if num.is_some() {
        return Ok(vec![format!("push {}", num.unwrap().raw_num)]);
    }
    let expr = m.exp.as_ref();
    if expr.is_none() {
        return Err(vec!["number or expression expected".into()]);
    }
    generate(expr.unwrap())
}
fn unary(u: &Unary) -> GenResult {
    match u.node.ope {
        None | Some(AddSub::Plus) => {
            return primary(&u.node);
        }
        _ => {
            let prim = primary(&u.node);
            if prim.is_err() {
                return prim;
            }
            let mut lines = vec![format!("push {}", 0)];
            lines.append(prim.unwrap().as_mut());
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
        let ope = u.ope.as_ref().unwrap();
        lines.append(second.unwrap().as_mut());
        lines.push("pop rdi".into());
        lines.push("pop rax".into());
        match ope {
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
        let ope = m.ope.as_ref().unwrap();
        lines.append(second.unwrap().as_mut());
        lines.push("pop rdi".into());
        lines.push("pop rax".into());
        match ope {
            AddSub::Plus => {
                lines.push("add rax,rdi".into());
            }
            AddSub::Minus => {
                lines.push("sub rax,rdi".into());
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
        let ope = a.ope.as_ref().unwrap();
        lines.append(second.unwrap().as_mut());
        lines.push("pop rdi".into());
        lines.push("pop rax".into());
        match ope {
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
                lines.push("setle al".into());
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

fn generate(expr: &Expr) -> GenResult {
    equality(&expr.node)
}
