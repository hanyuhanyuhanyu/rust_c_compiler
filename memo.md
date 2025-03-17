program = fdef*
  fdef =type ident"(" arg* ")" "{" stmt* "}"
  type="int" | "void"
stmt = if | for | while | "{" stmt* "}"| ("return ")? expr ";"
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
primary = num | ident | fcall | "(" expr ")"
  fcall=ident "(" arg* ")"
num=[0-9]+
ident=[a-zA-Z0-9_]+
