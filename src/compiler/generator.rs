use std::cmp;

use crate::compiler::consts::{IDENTITY_OFFSET, Register, register, sizeof};

use super::{
    consts::size_directive,
    node::{
        Add, AddSub, Assign, Block, Compare, Equality, Equals, Expr, Fcall, Fdef, For, If, Lvar,
        Mul, MulDiv, Primary, PrimaryNode, Program, PtrOpe, Relational, Statement, Typed, Unary,
        While,
    },
};
const PUSH_REF: &str = "push [rax]";
const PUSH_VAL: &str = "push rax";
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
const FARG_REGS: [Register; 6] = [
    Register::Di,
    Register::Si,
    Register::Dx,
    Register::Cx,
    Register::_8,
    Register::_9,
];
impl Generator<'_> {
    fn jump_label(&mut self) -> String {
        let label = self.jump_count.to_string();
        self.jump_count += 1;
        label
    }
    fn fcall(&mut self, f: &Fcall) -> GenResult {
        let mut lines = Vec::new();
        for e in f.args.iter().rev() {
            lines.extend(self.expr(&(&e.0, e.1.clone()))?);
        }

        // 6つまではレジスタ経由。rdi,rsi,rdx,rx,r8,r9の順。
        // 7つ目以降の引数はrbpの前に逆順で積む
        for i in 0..(cmp::min(f.args.len(), FARG_REGS.len())) {
            let r = FARG_REGS.iter().nth(i).unwrap();
            lines.extend(vec![format!("pop {}", register(0, r))]);
        }
        lines.push(format!("call {}", f.ident,));

        lines.push(PUSH_VAL.into());
        Ok(lines)
    }
    fn primary(&mut self, m: &Typed<Primary>) -> GenResult {
        match &m.0.node.0 {
            PrimaryNode::Num(n) => Ok(vec![format!("push {}", n.0)]),
            PrimaryNode::Expr(e) => self.expr(&(e, m.1.clone())),
            PrimaryNode::Fcall(f) => self.fcall(f),
            PrimaryNode::Lv(Lvar::Id(i)) => Ok(vec![
                "mov rax, rbp".into(),
                format!("sub rax, {}", i.offset),
                PUSH_REF.into(),
            ]),
        }
    }
    fn unary(&mut self, u: &Typed<Unary>) -> GenResult {
        match &u.0 {
            Unary::Ptr(p) => {
                let mut pri = self.unary(&p.unary)?;
                let last = pri.last();
                if last.is_none() {
                    return Err(vec![
                        "compiler buf: primary calc returned empty vector".into(),
                    ]);
                }
                match p.ope {
                    PtrOpe::Ref => Ok([pri, vec!["pop rax".into(), PUSH_REF.into()]].concat()),
                    PtrOpe::Deref => {
                        return if last.unwrap().eq(PUSH_REF) {
                            let len = pri.len() - 1;
                            pri[len] = PUSH_VAL.into();
                            Ok(pri)
                        } else {
                            Err(vec!["cannot handle multiple dereference".into()])
                        };
                    }
                }
            }
            Unary::Var(v) => {
                let pri = self.primary(&v.prim)?;
                match v.prim.0.ope {
                    None | Some(AddSub::Plus) => {
                        return Ok(pri);
                    }
                    _ => {
                        return Ok([
                            pri,
                            vec!["push 0", "pop rdi", "pop rax", "sub rdi, rax", "push rdi"]
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                        ]
                        .concat());
                    }
                }
            }
        }
    }
    fn mul(&mut self, m: &Typed<Mul>) -> GenResult {
        let first = self.unary(&m.0.first);
        if first.is_err() {
            return first;
        }
        if m.0.unarys.len() == 0 {
            return first;
        }
        let mut lines = first.unwrap();
        for u in m.0.unarys.iter() {
            let ope = u.0.ope();
            if ope.is_none() {
                return Err(vec!["operator expected".into()]);
            }
            let second = self.unary(u);
            if second.is_err() {
                return second;
            }
            lines.extend(second.unwrap());
            lines.push("pop rdi".into());
            lines.push("pop rax".into());
            match ope.as_ref().unwrap() {
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
    fn add(&mut self, a: &Typed<Add>) -> GenResult {
        let first = self.mul(&a.0.first);
        if first.is_err() {
            return first;
        }
        if a.0.muls.len() == 0 {
            return first;
        }
        let mut lines = first.unwrap();
        for m in a.0.muls.iter() {
            if m.0.ope.is_none() {
                return Err(vec!["operator expected".into()]);
            }
            let second = self.mul(m);
            if second.is_err() {
                return second;
            }
            lines.extend(second.unwrap());
            lines.push("pop rdi".into());
            lines.push("pop rax".into());
            match m.0.ope.as_ref().unwrap() {
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
    fn relational(&mut self, rel: &Typed<Relational>) -> GenResult {
        let first = self.add(&rel.0.first);
        if first.is_err() {
            return first;
        }
        if rel.0.adds.len() == 0 {
            return first;
        }
        let mut lines = first.unwrap();
        for a in rel.0.adds.iter() {
            if a.0.ope.is_none() {
                return Err(vec!["operator expected".into()]);
            }
            let second = self.add(a);
            if second.is_err() {
                return second;
            }
            lines.extend(second.unwrap());
            lines.push("pop rdi".into());
            lines.push("pop rax".into());
            //LInt型なのが問題？
            let ax = register(sizeof(&a.1), &Register::_Ax);
            let di = register(sizeof(&a.1), &Register::Di);
            match a.0.ope.as_ref().unwrap() {
                Compare::Lt => {
                    lines.push(format!("cmp {}, {}", ax, di));
                    lines.push("setl al".into());
                }
                Compare::Lte => {
                    lines.push(format!("cmp {}, {}", ax, di));
                    lines.push("setle al".into());
                }
                Compare::Gt => {
                    lines.push(format!("cmp {}, {}", di, ax));
                    lines.push("setl al".into());
                }
                Compare::Gte => {
                    lines.push(format!("cmp {}, {}", di, ax));
                    lines.push("setle al".into());
                }
            }
            lines.push("movzb rax, al".into());
            lines.push("push rax".into());
        }
        return Ok(lines);
    }
    fn equality(&mut self, eq: &Typed<Equality>) -> GenResult {
        let mut lines = self.relational(&eq.0.first)?;
        if eq.0.relationals.len() == 0 {
            return Ok(lines);
        }
        for rel in eq.0.relationals.iter() {
            if rel.0.ope.is_none() {
                return Err(vec!["operator expected".into()]);
            }
            let second = self.relational(&rel);
            if second.is_err() {
                return second;
            }
            let ope = rel.0.ope.as_ref().unwrap();
            lines.append(second.unwrap().as_mut());
            lines.push("pop rdi".into());
            lines.push("pop rax".into());
            lines.push("cmp rax, rdi".into()); // TODO
            match ope {
                Equals::Equal => lines.push("sete al".into()),
                Equals::NotEqual => lines.push("setne al".into()),
            }
            lines.push("movzb rax, al".into());
            lines.push("push rax".into());
        }
        return Ok(lines);
    }

    fn lvar(&mut self, e: &Lvar, ref_count: usize) -> GenResult {
        Ok(if ref_count <= 0 {
            match e {
                Lvar::Id(i) => vec![
                    "mov rax, rbp".into(),
                    format!("sub rax, {}", i.offset),
                    "push rax".into(),
                ],
            }
        } else {
            [
                self.lvar(e, ref_count - 1)?,
                vec!["pop rax".into(), PUSH_REF.into()],
            ]
            .concat()
        })
    }
    fn assign(&mut self, a: &Typed<&Assign>) -> GenResult {
        return match a {
            (Assign::Rv(r), _) => self.equality(&r.eq),
            (Assign::Asgn(a), _) => {
                let lvar = a.lvar.0.lvar();
                if lvar.is_none() {
                    return Err(vec!["expression canoot be assigned".into()]);
                }
                let (lv, _) = lvar.unwrap(); // 見直し
                let l = self.lvar(lv.0, lv.1)?;
                let mut r = self.expr(&(&a.rvar.0, a.rvar.1.clone()))?;
                r.extend(l);
                r.extend(vec![
                    "pop rax".into(),
                    "pop rdi".into(),
                    format!(
                        "mov {}[rax], {}",
                        size_directive(&a.lvar.1),
                        register(sizeof(&a.lvar.1), &Register::Di)
                    ),
                    "push rdi".into(),
                ]);
                Ok(r)
            }
        };
    }
    fn expr(&mut self, e: &Typed<&Expr>) -> GenResult {
        match e {
            (Expr::Asgn(ea), _) => self.assign(&(&ea.assign, e.1.clone())),
            (Expr::VarAsgn(def, assign), _) => {
                let mut l = vec![];
                if assign.is_none() {
                    l.push("push 0".into());
                }
                if assign.is_some() {
                    l.extend(
                        self.assign(&(assign.as_ref().unwrap(), assign.as_ref().unwrap().type_()))?,
                    );
                };
                l.push("pop rdi".into());
                for v in def.iter() {
                    l.extend(vec![
                        "mov rax, rbp".into(),
                        format!("sub rax, {}", v.offset),
                        format!(
                            "mov {}[rax], {}",
                            size_directive(&v._type_),
                            register(sizeof(&v._type_), &Register::Di)
                        ),
                    ]);
                }
                Ok(l)
            }
        }
    }
    fn for_(&mut self, f: &For) -> GenResult {
        let init = match &f.init {
            None => vec![],
            Some(e) => self.expr(&(&e.0, e.1.clone()))?,
        };
        let cond = match &f.cond {
            None => vec![],
            Some(e) => self.expr(&(&e.0, e.1.clone()))?,
        };
        let step = match &f.step {
            None => vec![],
            Some(e) => self.expr(&(&e.0, e.1.clone()))?,
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
        let cond = self.expr(&(&w.cond.0, w.cond.1.clone()))?;
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
        let cond = self.expr(&(&i.cond.0, i.cond.1.clone()))?;
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
            Statement::Nothing => Ok(vec![]),
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
                let lines = self.expr(&(&s.expr.0, s.expr.1.clone()))?;
                if s.expr.0.does_return() {
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
                    let r = FARG_REGS.iter().nth(i).unwrap();
                    vec![
                        "mov rax, rbp".into(),
                        format!("sub rax, {}", a.offset),
                        format!(
                            "mov {}[rax], {}",
                            size_directive(&a._type_),
                            register(sizeof(&a._type_), r)
                        ),
                    ]
                } else {
                    // 7つ目以降の引数はrbpの前に逆順で積まれている
                    let offset = (i + 1 - FARG_REGS.len()) * IDENTITY_OFFSET;
                    let sd = size_directive(&a._type_);
                    vec![
                        "mov rax, rbp".into(),
                        format!("add rax, {}", offset + IDENTITY_OFFSET), // リターンドレスの分で8余分に動かす
                        "mov rdi, [rax]".into(),
                        "mov rax, rbp".into(),
                        format!("sub rax, {}", a.offset),
                        format!(
                            "mov {}[rax], {}",
                            sd,
                            register(sizeof(&a._type_), &Register::Di)
                        ),
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
