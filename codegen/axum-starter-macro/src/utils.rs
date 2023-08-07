use heck::ToUpperCamelCase;
use syn::{spanned::Spanned, Expr};

pub(crate) fn snake_to_upper(src: &str) -> String {
    ToUpperCamelCase::to_upper_camel_case(src)
}

macro_rules! darling_err {
    ($provider:expr) => {
        match $provider {
            Ok(v) => v,
            Err(err) => return err.write_errors().into(),
        }
    };
}

pub fn check_callable_expr(expr: &Expr) -> Result<(), syn::Error> {
    if let Expr::Path(_) | Expr::Closure(_) = expr {
        Ok(())
    } else {
        Err(syn::Error::new(expr.span(), "Expect `Path` or `Closure`"))
    }
}

#[cfg(test)]
mod test {
    use crate::utils::snake_to_upper;

    #[test]
    fn test() {
        assert_eq!("Abc", snake_to_upper("abc"));
        assert_eq!("AbcCc", snake_to_upper("abc_cc"));
        assert_eq!("", snake_to_upper("_"));
        assert_eq!("AccBccDcc", snake_to_upper("acc_bcc_dcc"));
        assert_eq!("AccBccDcc", snake_to_upper("AccBccDcc"));
    }
}
