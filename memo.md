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
  fcall=ident "(" block? ("," block)* ")"
num=[0-9]+
ident=[a-zA-Z0-9_]+

何も考えずにretしているのでvoid funcを表現できない
未定義関数を呼んでもコンパイルエラーにならない　
arg listのtrailing commaを許したい