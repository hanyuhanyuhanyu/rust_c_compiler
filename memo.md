program = fdef*
fdef =type ident"(" arg? (","arg)* ")" block
arg=type ident
type="int" | "void"
block="{" stmt* "}"
stmt = if | for | while | block | ("return")? expr ";"
if="if (" expr ")" stmt ("else" stmt)?
for="for("expr?";"expr?";"expr?")" stmt
while="while("expr")" stmt
expr = assign
<!-- assign = equality("=" assign)? -->
assign = rvar | (lvar "=" expr)
rvar = equality
lvar = "*"* ident <!-- equalityのサブセットにする -->
equality =  relation (("==" | "!=") relational)*
relational = add (("<" | ">" | "<=" | ">=") add)*
add = mul ( "+" mul | "-" mul )*
mul  = unary ( "*" unary | "/" unary )*
unary = ("*" | "&") unary | ( "+" | "-" )? primary
primary = num | ident | fcall | "(" expr ")"  // void funcのことを考えるとこの定義だと困る未来が来そう
fcall=ident "(" expr? ("," expr)* ")"
num=[0-9]+
ident=identfirst(num | identfirst)*
identfirst=[a-zA-Z_]

何も考えずにretしているのでvoid funcを表現できない
未定義関数を呼んでもコンパイルエラーにならない　
arg listのtrailing commaを許したい
以下妄言。prologue/epilogueによってrspとrbpを適切に管理することで、関数内でどれだけスタックを汚しても他には影響しない。rspはスタックの参照位置を見ている。複文だと確かに複数個積んだりするけどargに渡すのはexprなので問題ない
  exprが常にスタックに積んでる。なので1exprが複数個スタックに積む恐れがある
    これだとfcall argをスタックに積む時、1引数が複数個スタックを積んでしまって期待する結果にならない
    これを、スタックを積む代わりにraxに入れて返す動きにしたい。exprを読んだ側が適宜pushしてあげるようにしたい
rbpの上に引数が積まれている前提で組んだけどこれだと一般のc言語の引数の取り扱いと互換性がないかも
  以下によると、6個以下はレジスタ、それ以上は引数の後ろからスタックに積んでおくそのようにするとよさげ
  https://qiita.com/hiro4669/items/348ba278aa31aa58fa95#abi%E3%81%AE%E7%A2%BA%E8%AA%8D 