use std::f32::consts::E;

use crate::parser::model::{Segment, Selector};
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer, State};
use crate::query::{Queried, Query};

impl Query for Segment {
    fn process<'a, 'b, T: Queryable>(
        &'b self,
        step: State<'a, T>,
        gas: &'b mut u32,
    ) -> Queried<State<'a, T>> {
        match self {
            Segment::Descendant(segment) => {
                segment.process(step.flat_map(process_descendant)?, gas)
            }
            Segment::Selector(selector) => selector.process(step, gas),
            Segment::Selectors(selectors) => process_selectors(step, selectors, gas),
        }
    }
}

fn process_selectors<'a, 'b, T: Queryable>(
    step: State<'a, T>,
    selectors: &Vec<Selector>,
    gas: &'b mut u32,
) -> Queried<State<'a, T>> {
    let mut reduced_state: Option<State<'_, T>> = None;
    for item in selectors.into_iter() {
        let new_state = item.process(step.clone(), gas)?;

        if let Some(state) = reduced_state {
            let f: State<'_, T> = state.reduce(new_state)?;
            reduced_state = Some(f);
        } else {
            reduced_state = Some(new_state);
        }
    }
    Ok(reduced_state.unwrap_or(step.root.into()))
}

fn process_descendant<T: Queryable>(data: Pointer<T>) -> Queried<Data<T>> {
    let result = if let Some(array) = data.inner.as_array() {
        let mut d2 = vec![];
        for (i, item) in array.iter().enumerate() {
            d2.push(Pointer::idx(item, data.path.clone(), i));
        }

        let d4 = Data::new_refs(d2).flat_map(process_descendant)?;

        Data::Ref(data.clone()).reduce(d4)

        // Data::Ref(data.clone()).reduce(
        //     Data::new_refs(
        //         array
        //             .iter()
        //             .enumerate()
        //             .map(|(i, elem)| Pointer::idx(elem, data.path.clone(), i))
        //             .collect(),
        //     )
        //     .flat_map(process_descendant),
        // )
    } else if let Some(object) = data.inner.as_object() {
        let mut d2 = vec![];
        for (key, value) in object.into_iter() {
            d2.push(Pointer::key(value, data.path.clone(), key));
        }
        Data::Ref(data.clone()).reduce(Data::new_refs(d2).flat_map(process_descendant)?)

        // Data::Ref(data.clone()).reduce(
        //     Data::new_refs(
        //         object
        //             .into_iter()
        //             .map(|(key, value)| Pointer::key(value, data.path.clone(), key))
        //             .collect(),
        //     )
        //     .flat_map(process_descendant),
        // )
    } else {
        Data::Nothing
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::parser::model::{Segment, Selector};
    use crate::query::state::{Pointer, State};
    use crate::query::Query;
    use serde_json::json;

    #[test]
    fn test_process_selectors() {
        let value = json!({"firstName": "John", "lastName" : "doe",});
        let segment = Segment::Selectors(vec![
            Selector::Name("firstName".to_string()),
            Selector::Name("lastName".to_string()),
        ]);
        let step = segment.process(State::root(&value), &mut 9999).unwrap();

        assert_eq!(
            step.ok_ref(),
            Some(vec![
                Pointer::new(&json!("John"), "$['firstName']".to_string()),
                Pointer::new(&json!("doe"), "$['lastName']".to_string())
            ])
        );
    }

    #[test]
    fn test_process_descendant() {
        let value = json!([{"name": "John"}, {"name": "doe"}]);
        let segment = Segment::Descendant(Box::new(Segment::Selector(Selector::Wildcard)));
        let step = segment.process(State::root(&value), &mut 9999).unwrap();

        assert_eq!(
            step.ok_ref(),
            Some(vec![
                Pointer::new(&json!({"name": "John"}), "$[0]".to_string()),
                Pointer::new(&json!({"name": "doe"}), "$[1]".to_string()),
                Pointer::new(&json!("John"), "$[0]['name']".to_string()),
                Pointer::new(&json!("doe"), "$[1]['name']".to_string()),
            ])
        );
    }

    #[test]
    fn test_process_descendant2() {
        let value = json!({"o": [0,1,[2,3]]});
        let segment = Segment::Descendant(Box::new(Segment::Selector(Selector::Index(1))));
        let step = segment.process(State::root(&value), &mut 9999).unwrap();

        assert_eq!(
            step.ok_ref(),
            Some(vec![
                Pointer::new(&json!(1), "$['o'][1]".to_string()),
                Pointer::new(&json!(3), "$['o'][2][1]".to_string()),
            ])
        );
    }
}
