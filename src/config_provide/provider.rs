pub trait Provider<'r, T: 'r> {
    fn provide(&'r self) -> T;
}

impl<'r, T> Provider<'r, &'r T> for T {
    fn provide(&'r self) -> &'r T {
        self
    }
}

macro_rules! group_provider {
    ($($args:ident),*$(,)?) => {

        impl<'r, C,$($args),*> Provider<'r, ($($args,)*)> for C
        where
            $(
                $args: 'r,
                C: Provider<'r,$args>,
            )*
        {
            #[allow(clippy::unused_unit)]
            fn provide(&'r self) -> ($($args,)*) {
                (
                    $(
                        <C as Provider<'r,$args>>::provide(self),
                    )*
                )
            }
        }
    };
}

group_provider!();
group_provider!(T1);
group_provider!(T1, T2);
group_provider!(T1, T2, T3);
group_provider!(T1, T2, T3, T4);
group_provider!(T1, T2, T3, T4, T5);
group_provider!(T1, T2, T3, T4, T5, T6);
group_provider!(T1, T2, T3, T4, T5, T6, T7);
group_provider!(T1, T2, T3, T4, T5, T6, T7, T8);
group_provider!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
group_provider!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
