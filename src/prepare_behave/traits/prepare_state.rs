use crate::prepare_behave::effect_collectors::state_collector::StateCollector;

/// Prepare for Global State
///
/// for instance the Connection Pool of Database

pub trait PrepareStateEffect: 'static {
    fn take_state(self, states: &mut StateCollector);
}

macro_rules! state_effect {
    ($($id:ident),* $(,)?) => {
        impl<$($id),*> PrepareStateEffect for ($($id,)*)
        where
            $(
                $id: PrepareStateEffect,
            )*
        {
            #[allow(non_snake_case,unused_variables)]
            fn take_state(self, states: &mut StateCollector) {
                let ($($id,)*) = self;

                $(
                    $id.take_state(states);
                )*

            }
        }
    };
}

state_effect!();
state_effect!(T1);
state_effect!(T1, T2);
state_effect!(T1, T2, T3);
state_effect!(T1, T2, T3, T4);
state_effect!(T1, T2, T3, T4, T5);
state_effect!(T1, T2, T3, T4, T5, T6);
state_effect!(T1, T2, T3, T4, T5, T6, T7);
state_effect!(T1, T2, T3, T4, T5, T6, T7, T8);
