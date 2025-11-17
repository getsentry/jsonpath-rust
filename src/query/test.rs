use crate::parser::model::Test;
use crate::query::queryable::Queryable;
use crate::query::state::State;
use crate::query::{Queried, Query};

impl Query for Test {
    fn process<'a, 'b, T: Queryable>(
        &'b self,
        state: State<'a, T>,
        gas: &'b mut u32,
    ) -> Queried<State<'a, T>> {
        match self {
            Test::RelQuery(segments) => segments.process(state, gas),
            Test::AbsQuery(jquery) => jquery.process(state.shift_to_root(), gas),
            Test::Function(tf) => tf.process(state, gas),
        }
    }
}
