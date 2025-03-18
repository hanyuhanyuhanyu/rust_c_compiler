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
assign = equality("=" assign)?
equality =  relation (("==" | "!=") relational)*
relational = add (("<" | ">" | "<=" | ">=") add)*
add = mul ( "+" mul | "-" mul )*
mul  = unary ( "*" unary | "/" unary )*
unary = ( "+" | "-" )? primary
primary = num | ident | fcall | "(" expr ")"  // void funcのことを考えるとこの定義だと困る未来が来そう
fcall=ident "(" expr? ("," expr)* ")"
num=[0-9]+
ident=[a-zA-Z0-9_]+

何も考えずにretしているのでvoid funcを表現できない
未定義関数を呼んでもコンパイルエラーにならない　
arg listのtrailing commaを許したい
rbpの上に引数が積まれている前提で組んだけどこれだと一般のc言語の引数の取り扱いと互換性がないかも
  以下によると、6個以下はレジスタ、それ以上は引数の後ろからスタックに積んでおくそのようにするとよさげ
  https://qiita.com/hiro4669/items/348ba278aa31aa58fa95#abi%E3%81%AE%E7%A2%BA%E8%AA%8D 