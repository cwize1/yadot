%start Root
%%
Root -> Result<&'input str, ()>:
    '$' '{{' Expr '}}' {
        $3
    } ;

Expr -> Result<&'input str, ()>:
    'STRING' {
        let v = $1.map_err(|_| ())?;
        Ok($lexer.span_str(v.span()))
    } ;

%%
