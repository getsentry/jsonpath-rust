use crate::parser::model::{JpQuery, Segment};
use crate::query::queryable::Queryable;
use crate::query::state::State;
use crate::query::{Queried, Query};

impl Query for JpQuery {
    fn process<'a, 'b, T: Queryable>(
        &'b self,
        state: State<'a, T>,
        gas: &'b mut u32,
    ) -> Queried<State<'a, T>> {
        self.segments.process(state, gas)
    }
}

impl Query for Vec<Segment> {
    fn process<'a, 'b, T: Queryable>(
        &'b self,
        state: State<'a, T>,
        gas: &'b mut u32,
    ) -> Queried<State<'a, T>> {
        let mut state = state;
        for segment in self.iter() {
            state = segment.process(state, gas)?;
        }
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::model::{JpQuery, Segment, Selector};
    use crate::query::state::{Data, Pointer, State};
    use crate::query::Query;
    use serde_json::json;

    #[test]
    fn test_process() {
        let value = json!({
          "result": [
            {
              "message": "Hello, Emmy! Your order number is: #100",
              "phoneNumber": "255-301-9429",
              "phoneVariation": "+90 398 588 10 73",
              "status": "active",
              "name": {
                "first": "Blaise",
                "middle": "Kyle",
                "last": "Fadel"
              }
            }
          ]
        });

        let query = JpQuery::new(vec![
            Segment::Selector(Selector::Name("result".to_string())),
            Segment::Selector(Selector::Index(0)),
            Segment::Selector(Selector::Name("name".to_string())),
            Segment::Selector(Selector::Name("first".to_string())),
        ]);

        let state = State::data(&value, Data::new_ref(Pointer::new(&value, "$".to_string())));

        let result = query.process(state, &mut 9999).unwrap();
        assert_eq!(
            result.ok_ref(),
            Some(vec![Pointer::new(
                &json!("Blaise"),
                "$['result'][0]['name']['first']".to_string()
            )])
        );
    }
}
