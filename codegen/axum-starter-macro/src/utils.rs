pub(crate) fn snake_to_upper(src: &str) -> String {
    let mut st = String::with_capacity(src.len());
    let chars = src.chars();

    chars.fold((&mut st, true), |(st, need_upper), ch| {
        if ch == '_' {
            (st, true)
        } else {
            let ch = if need_upper {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            st.push(ch);
            (st, false)
        }
    });

    st
}

macro_rules! darling_err {
    ($provider:expr) => {
        match $provider {
            Ok(v) => v,
            Err(err) => return err.write_errors().into(),
        }
    };
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
