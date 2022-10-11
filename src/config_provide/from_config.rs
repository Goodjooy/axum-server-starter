pub trait FromConfig<'r, C>: Sized {
    fn from_config(config: &'r C) -> Self;
}

macro_rules! group_from_config {
    ($($args:ident),*$(,)?) => {
        impl<'r, Config, $($args),*> FromConfig<'r, Config> for ($($args,)*)
        where
        $(
            $args: FromConfig<'r, Config>,
        )*
        {
            #[allow(unused_variables)]
            #[allow(clippy::unused_unit)]
            fn from_config(config: &'r Config) -> Self {
                (
                    $(
                        <$args as FromConfig<'r,Config>>::from_config(config)
                    ,)
                *)
            }
        }
    };
}

group_from_config!();
group_from_config!(A);
group_from_config!(A, B);
group_from_config!(A, B, C);
group_from_config!(A, B, C, D);
group_from_config!(A, B, C, D, E);
group_from_config!(A, B, C, D, E, F);
group_from_config!(A, B, C, D, E, F, G);
group_from_config!(A, B, C, D, E, F, G, H);
group_from_config!(A, B, C, D, E, F, G, H, I,);
group_from_config!(A, B, C, D, E, F, G, H, I, J);
group_from_config!(A, B, C, D, E, F, G, H, I, J, K);
