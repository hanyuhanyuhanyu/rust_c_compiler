program = (stmt)*
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
primary = num | ident | "(" expr ")"
num=[0-9]+
ident=[a-z]+
