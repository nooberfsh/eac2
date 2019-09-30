use super::Terminal;

struct Context{
    tokens: Vec<Terminal>,
    idx: usize,
    eof: Terminal,
}

pub struct Error;

type Result = std::result::Result<(), Error>;

pub fn parse(tokens: Vec<Terminal>) -> Result {
    let mut ctx = Context::new(tokens)   ;
    start(&mut ctx)
}

fn start(ctx: &mut Context) -> Result {
    expr(ctx)?;
    if ctx.current().is_eof() {
        Ok(())
    } else {
        Err(Error)
    }
}

fn expr(ctx: &mut Context) -> Result {
    term(ctx)?;
    expr__(ctx)
}

fn expr__(ctx: &mut Context) -> Result {
    let t = &ctx.current().0;
    if t == "+" || t == "-" {
        ctx.forward();
        term(ctx)?;
        expr__(ctx)?;
    }
    Ok(())
}

fn term(ctx: &mut Context) -> Result {
    factor(ctx)?;
    term__(ctx)
}

fn factor(ctx: &mut Context) -> Result {
    let t = &ctx.current().0;
    if t == "num" || t == "name" {
        ctx.forward();
        Ok(())
    }  else if t == "(" {
        ctx.forward();
        expr(ctx)?;
        if ctx.current().0 == ")" {
            ctx.forward();
            Ok(())
        } else {
            Err(Error)
        }
    }
    else {
        Err(Error)
    }
}

fn term__(ctx: &mut Context) -> Result {
    let t = &ctx.current().0;
    if t == "*" || t == "/" {
        ctx.forward();
        factor(ctx)?;
        term__(ctx)?;
    }
    Ok(())
}

impl Context {
    fn new(tokens: Vec<Terminal>) -> Self {
        Context {
            tokens: tokens,
            idx: 0,
            eof: Terminal::eof(),
        }
    }

    fn forward(&mut self) {
        if self.idx != self.tokens.len() {
            self.idx += 1;
        }
    }

    fn current(&self) -> &Terminal {
        if self.idx == self.tokens.len() {
            &self.eof
        } else {
            &self.tokens[self.idx]
        }
    }
}
