use crate::compiler::consts::IDENTITY_OFFSET;
use std::cmp;

use super::node::{
    Add, AddSub, Assign, Block, Compare, Equality, Equals, Expr, Fcall, Fdef, For, If, Lvar, Mul,
    MulDiv, Primary, PrimaryNode, Program, Relational, Statement, Unary, While,
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
struct Generator<'a> {
    p: &'a Program,
    jump_count: usize,
}
const FARG_REGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
impl Generator<'_> {
    fn jump_label(&mut self) -> String {
        let label = self.jump_count.to_string();
        self.jump_count += 1;
        label
    }
    fn fcall(&mut self, f: &Fcall) -> GenResult {
        let mut lines = Vec::new();
        for e in f.args.iter().rev() {
            lines.extend(self.expr(e)?);
        }

        // 6つまではレジスタ経由。rdi,rsi,rdx,rx,r8,r9の順。
        // 7つ目以降の引数はrbpの前に逆順で積む
        for i in 0..(cmp::min(f.args.len(), FARG_REGS.len())) {
            let register = FARG_REGS.iter().nth(i).unwrap();
            lines.extend(vec![format!("pop {}", register)]);
        }
        lines.push(format!("call {}", f.ident,));

        lines.push("push rax".into());
        Ok(lines)
    }
    fn primary(&mut self, m: &Primary) -> GenResult {
        match &m.node {
            PrimaryNode::Num(n) => Ok(vec![format!("push {}", n)]),
            PrimaryNode::Expr(e) => self.expr(&e),
            PrimaryNode::Fcall(f) => self.fcall(f),
            PrimaryNode::Lv(Lvar::Id(i)) => Ok(vec![
                "mov rax, rbp".into(),
                format!("sub rax, {}", i.offset),
                "push [rax]".into(),
            ]),
        }
    }
    fn unary(&mut self, u: &Unary) -> GenResult {
        match u.prim.ope {
            None | Some(AddSub::Plus) => {
                return self.primary(&u.prim);
            }
            _ => {
                let prim = self.primary(&u.prim)?;
                Ok([
                    prim,
                    vec!["push 0", "pop rdi", "pop rax", "sub rdi, rax", "push rdi"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ]
                .concat())
            }
        }
    }
    fn mul(&mut self, m: &Mul) -> GenResult {
        let first = self.unary(&m.first);
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
            let second = self.unary(u);
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
    fn add(&mut self, a: &Add) -> GenResult {
        let first = self.mul(&a.first);
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
            let second = self.mul(m);
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
    fn relational(&mut self, rel: &Relational) -> GenResult {
        let first = self.add(&rel.first);
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
            let second = self.add(a);
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
    fn equality(&mut self, eq: &Equality) -> GenResult {
        let mut lines = self.relational(&eq.first)?;
        if eq.relationals.len() == 0 {
            return Ok(lines);
        }
        for rel in eq.relationals.iter() {
            if rel.ope.is_none() {
                return Err(vec!["operator expected".into()]);
            }
            let second = self.relational(rel);
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

    fn lvar(&mut self, e: &Lvar) -> GenResult {
        match e {
            Lvar::Id(i) => Ok(vec![
                "mov rax, rbp".into(),
                format!("sub rax, {}", i.offset),
                "push rax".into(),
            ]),
        }
    }
    fn assign(&mut self, a: &Assign) -> GenResult {
        return match a {
            Assign::Rv(r) => self.equality(&r.eq),
            Assign::Asgn(a) => {
                let lvar = a.lvar.lvar();
                if lvar.is_none() {
                    return Err(vec!["expression canoot be assigned".into()]);
                }
                let l = self.lvar(lvar.unwrap())?;
                let mut r = self.expr(&a.rvar)?;
                r.extend(l);
                r.extend(vec![
                    "pop rax".into(),
                    "pop rdi".into(),
                    "mov [rax], rdi".into(),
                    "push rdi".into(),
                ]);
                Ok(r)
            }
        };
    }
    fn expr(&mut self, e: &Expr) -> GenResult {
        self.assign(&e.assign)
    }
    fn for_(&mut self, f: &For) -> GenResult {
        let init = match &f.init {
            None => vec![],
            Some(e) => self.expr(e)?,
        };
        let cond = match &f.cond {
            None => vec![],
            Some(e) => self.expr(e)?,
        };
        let step = match &f.step {
            None => vec![],
            Some(e) => self.expr(e)?,
        };
        let stmt = self.stmt(&f.stmt)?;
        let start_label = format!(".ForStart{}", self.jump_label());
        let end_label = format!(".EndStart{}", self.jump_label());
        Ok([
            init,
            vec![start_label.clone() + ":"],
            cond,
            vec![
                "pop rax".into(),
                "cmp rax, 0".into(),
                "je ".to_string() + &end_label,
            ],
            stmt,
            step,
            vec!["jmp ".to_string() + &start_label, end_label + ":"],
        ]
        .concat())
    }
    fn while_(&mut self, w: &While) -> GenResult {
        let cond = self.expr(&w.cond)?;
        let start_label = format!(".WhileStart{}", self.jump_label());
        let end_label = format!(".WhileEnd{}", self.jump_label());
        let stmt = self.stmt(&w.stmt)?;
        Ok([
            vec![start_label.clone() + ":"],
            cond,
            vec![
                "pop rax".into(),
                "cmp rax, 0".into(),
                "je ".to_string() + &end_label,
            ],
            stmt,
            vec!["jmp ".to_string() + &start_label, end_label + ":"],
        ]
        .concat())
    }
    fn if_(&mut self, i: &If) -> GenResult {
        let cond = self.expr(&i.cond)?;
        let end_label = format!(".IfEnd{}", self.jump_label());
        let stmt = self.stmt(&i.stmt)?;
        if i.else_.is_none() {
            return Ok([
                cond,
                vec![
                    "pop rax".into(),
                    "cmp rax, 0".into(),
                    "je ".to_string() + &end_label,
                ],
                stmt,
                vec![end_label + ":"],
            ]
            .concat());
        }
        let else_ = self.stmt(i.else_.as_ref().unwrap())?;
        let else_label = format!(".IfElse{}", self.jump_label());
        Ok([
            cond,
            vec![
                "pop rax".into(),
                "cmp rax, 0".into(),
                "je ".to_string() + &else_label,
            ],
            stmt,
            vec!["jmp ".to_string() + &end_label, else_label + ":"],
            else_,
            vec![end_label + ":"],
        ]
        .concat())
    }
    fn stmt(&mut self, stmt: &Statement) -> GenResult {
        match stmt {
            Statement::If(i) => self.if_(i),
            Statement::While(w) => self.while_(w),
            Statement::For(f) => self.for_(f),
            Statement::MStmt(ms) => ms
                .stmts
                .iter()
                .map(|f| self.stmt(f))
                .reduce(|a, b| concat(a, b))
                .unwrap_or(Ok(vec![])),
            Statement::Stmt(s) => {
                let lines = self.expr(&s.expr)?;
                if s.expr.ret {
                    Ok([lines, self.epilogue()?].concat())
                } else {
                    Ok(lines)
                }
            }
        }
    }
    fn block(&mut self, b: &Block) -> GenResult {
        let mut lines = Vec::new();
        for s in b.stmts.iter() {
            let ls = self.stmt(s)?;
            lines.extend(ls);
        }
        Ok(lines)
    }
    fn prologue(&mut self, f: &Fdef) -> GenResult {
        // 引数を頭から順に入れたらstackには逆順に入っているはず
        let args: Vec<Vec<String>> = f
            .args
            .iter()
            .enumerate()
            .map(|(i, a)| {
                // 6つまではレジスタ経由。rdi,rsi,rdx,rx,r8,r9の順。
                if i < FARG_REGS.len() {
                    let register = FARG_REGS.iter().nth(i).unwrap();
                    vec![
                        "mov rax, rbp".into(),
                        format!("sub rax, {}", a.offset),
                        format!("mov [rax], {}", register),
                    ]
                } else {
                    // 7つ目以降の引数はrbpの前に逆順で積まれている
                    let offset = (i + 1 - FARG_REGS.len()) * IDENTITY_OFFSET;
                    vec![
                        "mov rax, rbp".into(),
                        format!("add rax, {}", offset + IDENTITY_OFFSET), // リターンドレスの分で8余分に動かす
                        "mov rdi, [rax]".into(),
                        "mov rax, rbp".into(),
                        format!("sub rax, {}", a.offset),
                        "mov [rax], rdi".into(),
                    ]
                }
            })
            .collect();
        Ok(vec![
            vec![
                format!("{}:", f.ident),
                "push rbp #prlg ->".into(),
                "mov rbp, rsp".into(),
                format!("sub rsp, {} {}", f.required_memory, "#<- prlg"),
            ],
            args.concat(),
        ]
        .concat())
    }
    fn epilogue(&mut self) -> GenResult {
        Ok(vec![
            "pop rax #eplg ->".into(),
            "mov rsp, rbp".into(),
            "pop rbp".into(),
            "ret #<- eplg".into(),
        ])
    }
    fn fdef(&mut self) -> GenResult {
        let mut genr = Ok(Vec::new());
        for f in self.p.fdefs.iter() {
            genr = concat(
                genr,
                concat_multi(&[self.prologue(f), self.block(&f.fimpl), self.epilogue()]),
            );
        }
        genr
    }
    fn generate(&mut self) -> GenResult {
        self.fdef()
    }
}
pub fn generate(p: &Program) -> GenResult {
    Generator {
        p: p,
        jump_count: 0,
    }
    .generate()
}
