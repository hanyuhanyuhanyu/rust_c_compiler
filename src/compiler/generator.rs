use std::{cmp, collections::HashMap};

use crate::compiler::consts::{IDENTITY_OFFSET, Register, register};

use super::{
    consts::{LEFT_VALUE_IS_NOT_ASSIGNABLE, size_directive},
    node::{
        Add, AddSub, Assign, Block, Compare, Equality, Equals, Expr, Fcall, Fdef, For, If, Lvar,
        Mul, MulDiv, Primary, PrimaryNode, Program, PtrOpe, Relational, Statement, Typed, Unary,
        VarDef, While,
    },
    type_::Type,
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
    _array_size: HashMap<(&'a String, usize), Vec<usize>>,
}
const FARG_REGS: [Register; 6] = [
    Register::Di,
    Register::Si,
    Register::Dx,
    Register::Cx,
    Register::_8,
    Register::_9,
];
fn push_ref(t: &Type) -> Vec<String> {
    vec![
        format!(
            "mov {}, {}[{}] # {:?}",
            register(t.sizeof(), &Register::_Ax),
            size_directive(&t),
            register(8, &Register::_Ax),
            t
        ),
        PUSH_VAL.into(),
    ]
}
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
    fn primary(&mut self, m: &Typed<Primary>, arr: &Vec<Typed<Expr>>, is_rvar: bool) -> GenResult {
        if !is_rvar && !m.0.is_lvar() {
            return Err(vec![LEFT_VALUE_IS_NOT_ASSIGNABLE.into()]);
        }
        let mut lines = match &m.0.node.0 {
            PrimaryNode::Num(n) => Ok(vec![format!("push {}", n.0)]),
            PrimaryNode::Expr(e) => self.expr(&(e, m.1.clone())), // TODO これだとこれに直接配列アクセスしようとしたら困りそう。これの戻り値がLvであるとわからないと難しい。右式の変数がアドレスからその内部の値に姿を変えるのは代入演算子('=')によるものだと解釈するほうが良いのでは？　ひとまずExprに対する配列アクセスはサポートしない
            PrimaryNode::Fcall(f) => self.fcall(f),
            PrimaryNode::Lv(Lvar::Id(i)) => {
                let mut lines = vec!["mov rax, rbp".into(), format!("sub rax, {}", i.offset)];
                if is_rvar && arr.len() == 0 {
                    lines.extend(push_ref(&i._type_));
                } else {
                    lines.push(PUSH_VAL.into());
                };
                Ok(lines)
            }
        }?;
        if arr.len() == 0 {
            return Ok(lines);
        }
        // TODO 配列アクセスしてよいやつかどうかチェック
        let (t, depth) = match &m.1 {
            Type::Array(b) => {
                let t = &b.0;
                let depth = b.1;
                if depth < arr.len() {
                    return Err(vec![format!(
                        "this array has {} dimension, cannot access {} dimension",
                        depth,
                        arr.len()
                    )]);
                }
                (t, depth)
            }
            _ => return Err(vec!["this node is not array".into()]),
        };
        let item_size = t.sizeof_item();
        lines.push("push 0 # arr start".into()); // 配列のオフセット
        for (ind, a) in arr.iter().enumerate() {
            lines.extend(self.expr(&(&a.0, a.1.clone()))?);
            lines.push(format!("#{:?}", a.0));
            // TODO: 配列の領域外アクセスチェック
            lines.extend(vec![
                format!("pop rax # {}", ind), // 算出した要素数
                "pop rdi".into(),             // 配列のオフセット
                "pop rsi".into(),             // 配列のアドレス
                format!("mov rdx, rsi"),
                if ind == depth - 1 {
                    format!("mov rdx, 1",) // 多次元配列の端っこなら固定値
                } else {
                    format!(
                        "add rdx, 0x{:X}\nmov rdx, [rdx]",
                        (ind + 2) * IDENTITY_OFFSET //(ind + 2)で正しい。配列の実体のポインタの直上は配列全体の大きさを格納している。その一つ上が、1次元目の配列1つあたりのメモリの大きさを表している
                    )
                },
                "imul rax, rdx".into(),
                "add rdi, rax".into(),
                "push rsi".into(),
                "push rdi".into(),
            ]);
        }
        lines.extend(vec![
            "pop rdi # arr end".into(),
            "pop rax".into(),
            format!("imul rdi, 0x{:X}", item_size),
            "sub rax, rdi".into(),
            if is_rvar {
                PUSH_REF.into()
            } else {
                PUSH_VAL.into()
            },
        ]);

        Ok(lines)
    }
    fn unary(&mut self, u: &Typed<Unary>, is_rvar: bool) -> GenResult {
        if !is_rvar && !u.0.is_lvar() {
            return Err(vec![LEFT_VALUE_IS_NOT_ASSIGNABLE.into()]);
        }
        match &u.0 {
            Unary::Ptr(p) => {
                let mut pri = self.unary(&p.unary, is_rvar)?;
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
                let pri = self.primary(&v.prim, &v._arrs, is_rvar)?;
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
    fn mul(&mut self, m: &Typed<Mul>, is_rvar: bool) -> GenResult {
        if !is_rvar && !m.0.is_lvar() {
            return Err(vec![LEFT_VALUE_IS_NOT_ASSIGNABLE.into()]);
        }
        let first = self.unary(&m.0.first, is_rvar);
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
            let second = self.unary(u, is_rvar);
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
    fn add(&mut self, a: &Typed<Add>, is_rvar: bool) -> GenResult {
        if !is_rvar && !a.0.is_lvar() {
            return Err(vec![LEFT_VALUE_IS_NOT_ASSIGNABLE.into()]);
        }
        let first = self.mul(&a.0.first, is_rvar);
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
            let second = self.mul(m, is_rvar);
            if second.is_err() {
                return second;
            }
            lines.extend(second.unwrap());
            lines.push("pop rdi".into());
            lines.extend(a.1.when_addsub("rdi".into())); // FIXME: 型によって加減算のルールを指定したい。とても場当たり的なコード
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
    fn relational(&mut self, rel: &Typed<Relational>, is_rvar: bool) -> GenResult {
        if !is_rvar && !rel.0.is_lvar() {
            return Err(vec![LEFT_VALUE_IS_NOT_ASSIGNABLE.into()]);
        }
        let first = self.add(&rel.0.first, is_rvar);
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
            let second = self.add(a, is_rvar);
            if second.is_err() {
                return second;
            }
            lines.extend(second.unwrap());
            lines.push("pop rdi".into());
            lines.push("pop rax".into());
            //LInt型なのが問題？
            let ax = register(a.1.sizeof(), &Register::_Ax);
            let di = register(a.1.sizeof(), &Register::Di);
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
    fn equality(&mut self, eq: &Typed<Equality>, is_rvar: bool) -> GenResult {
        if !is_rvar && !eq.0.is_lvar() {
            return Err(vec![LEFT_VALUE_IS_NOT_ASSIGNABLE.into()]);
        }
        let mut lines = self.relational(&eq.0.first, is_rvar)?;
        if eq.0.relationals.len() == 0 {
            return Ok(lines);
        }
        for rel in eq.0.relationals.iter() {
            if rel.0.ope.is_none() {
                return Err(vec!["operator expected".into()]);
            }
            let second = self.relational(&rel, is_rvar);
            if second.is_err() {
                return second;
            }
            let ope = rel.0.ope.as_ref().unwrap();
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

    fn assign(&mut self, a: &Typed<&Assign>) -> GenResult {
        return match a {
            (Assign::Rv(r), _) => self.equality(&r.eq, true),
            (Assign::Asgn(a), _) => {
                let l = self.equality(&a.lvar, false)?;

                let mut r = self.expr(&(&a.rvar.0, a.rvar.1.clone()))?;
                r.extend(l);
                r.extend(vec![
                    "pop rax".into(),
                    "pop rdi".into(),
                    format!(
                        "mov {}[rax], {} # {:?}",
                        size_directive(&a.rvar.1),
                        register(a.rvar.1.sizeof(), &Register::Di),
                        a.rvar.1.clone()
                    ),
                    "push rdi".into(),
                ]);
                Ok(r)
            }
        };
    }
    fn vardef(&mut self, v: &VarDef) -> GenResult {
        if v._arrs.len() == 0 {
            return Ok(vec![
                "mov rax, rbp".into(),
                format!("sub rax, {}", v.offset),
                format!(
                    "mov {}[rax], {}",
                    size_directive(&v.type_),
                    register(v.type_.sizeof(), &Register::Di)
                ),
            ]);
        }
        //r15に配列全体のバイト数を持っておく
        let mut lines = vec![format!("mov r15, 0x{} # arr def start", 1)];
        let len = v._arrs.len();
        for (ind, a) in v._arrs.iter().rev().enumerate() {
            lines.extend(self.expr(&(&a.0, a.1.clone()))?);
            // 各配列の大きさを配列自体のポインタの上に確保。浅い順からポインタに近い位置に置く
            lines.extend(vec![
                "pop rdi".into(),
                "imul r15, rdi".into(), // サイズ大きくする
                "mov rax, rbp".into(),
                format!("sub rax, 0x{:X}", v.offset - (len - ind) * IDENTITY_OFFSET),
                "mov [rax], r15".into(),
            ])
        }
        // 配列自体を指すポイントを確保
        lines.push("mov rax, rbp # arr ptr".into());
        lines.push(format!("sub rax, 0x{:X}", v.offset));
        lines.push("sub rsp, 0x8".into()); // TODO 多分rsp無意味に押し下げすぎ。
        lines.push("mov [rax], rsp".into());
        lines.push(format!("imul r15, 0x{:X}", v.type_.sizeof_item()));
        lines.push("sub rsp, r15".into()); // 配列全体のメモリを確保
        lines.push("sub rsp, 0x8".into());
        lines.push("mov r15, 0x0".into()); // r15後片付け
        Ok(lines)
    }
    fn expr(&mut self, e: &Typed<&Expr>) -> GenResult {
        match e {
            (Expr::Asgn(ea), _) => self.assign(&(&ea.assign, e.1.clone())),
            (Expr::VarAsgn(def, assign), _) => {
                let mut l = vec![];
                //0で初期化
                if assign.is_none() {
                    l.push("push 0".into());
                } else {
                    l.extend(
                        self.assign(&(assign.as_ref().unwrap(), assign.as_ref().unwrap().type_()))?,
                    );
                };
                l.push("pop rdi".into());
                for v in def.iter() {
                    l.extend(self.vardef(v)?);
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
                            size_directive(&a.type_),
                            register(a.type_.sizeof(), r)
                        ),
                    ]
                } else {
                    // 7つ目以降の引数はrbpの前に逆順で積まれている
                    let offset = (i + 1 - FARG_REGS.len()) * IDENTITY_OFFSET;
                    let sd = size_directive(&a.type_);
                    vec![
                        "mov rax, rbp".into(),
                        format!("add rax, {}", offset + IDENTITY_OFFSET), // リターンドレスの分で8余分に動かす
                        "mov rdi, [rax]".into(),
                        "mov rax, rbp".into(),
                        format!("sub rax, {}", a.offset),
                        format!(
                            "mov {}[rax], {}",
                            sd,
                            register(a.type_.sizeof(), &Register::Di)
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
        _array_size: HashMap::new(),
    }
    .generate()
}
