use crate::parser::model::Filter;
use crate::query::gas::use_gas;
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer, State};
use crate::query::{Queried, Query};

impl Query for Filter {
    fn process<'a, 'b, T: Queryable>(
        &'b self,
        state: State<'a, T>,
        gas: &'b mut u32,
    ) -> Queried<State<'a, T>> {
        let root = state.root;
        state.flat_map(|p| {
            let result = if p.is_internal() {
                Data::Value(self.filter_item(p, root, gas)?.into())
            } else if let Some(items) = p.inner.as_array() {
                let mut data = vec![];
                for (idx, item) in items.into_iter().enumerate() {
                    if self.filter_item(Pointer::empty(item), root, gas)? {
                        data.push(Pointer::idx(item, p.path.clone(), idx));
                    }
                }
                Data::Refs(data)
            } else if let Some(items) = p.inner.as_object() {
                let mut data = vec![];
                for (key, item) in items.into_iter() {
                    if self.filter_item(Pointer::empty(item), root, gas)? {
                        data.push(Pointer::key(item, p.path.clone(), key));
                    }
                }
                Data::Refs(data)
            } else {
                use_gas(gas, 1)?;
                Data::Nothing
            };
            Ok(result)
        })
    }
}

impl Filter {
    fn process_elem<'a, 'b, T: Queryable>(
        &'b self,
        state: State<'a, T>,
        gas: &'b mut u32,
    ) -> Queried<State<'a, T>> {
        let mut process_cond = |filter: &Filter| -> Queried<bool> {
            let r = filter
                .process(state.clone(), gas)?
                .ok_val()
                .and_then(|v| v.as_bool())
                .unwrap_or_default();
            Ok(r)
        };
        match self {
            Filter::Or(ors) => {
                let mut result = false;
                for or in ors {
                    if process_cond(or)? {
                        result = true;
                        break;
                    }
                }
                Ok(State::bool(result, state.root))
            }
            Filter::And(ands) => {
                let mut result = true;
                for and in ands {
                    if !process_cond(and)? {
                        result = false;
                        break;
                    }
                }
                Ok(State::bool(result, state.root))
            }
            Filter::Atom(atom) => atom.process(state, gas),
        }
    }

    fn filter_item<'a, 'b, T: Queryable>(
        &'b self,
        item: Pointer<'a, T>,
        root: &T,
        gas: &'b mut u32,
    ) -> Queried<bool> {
        let result = self
            .process_elem(State::data(root, Data::Ref(item.clone())), gas)?
            .ok_val()
            .and_then(|v| v.as_bool())
            .unwrap_or_default();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::query::js_path;
    use serde_json::json;

    #[test]
    fn smoke_ok() {
        let json = json!({"a" : [1,2,3]});

        assert_eq!(
            js_path("$.a[? @ > 1]", &json, 9999),
            Ok(vec![
                (&json!(2), "$['a'][1]".to_string()).into(),
                (&json!(3), "$['a'][2]".to_string()).into(),
            ])
        );
    }

    #[test]
    fn existence() {
        let json = json!({
          "a": {
            "a":{"b":1},
            "c": {
              "b": 2
            },
            "d": {
              "b1": 3
            }
          }
        });
        assert_eq!(
            js_path("$.a[?@.b]", &json, 9999),
            Ok(vec![
                (&json!({"b":1}), "$['a']['a']".to_string()).into(),
                (&json!({"b":2}), "$['a']['c']".to_string()).into(),
            ])
        );
    }

    #[test]
    fn existence_or() {
        let json = json!({
          "a": {
            "a":{"b":1},
            "c": {
              "b": 2
            },
            "d": {
              "b1": 3
            },
            "e": {
              "b2": 3
            }
          }
        });
        assert_eq!(
            js_path("$.a[?@.b || @.b1]", &json, 9999),
            Ok(vec![
                (&json!({"b":1}), "$['a']['a']".to_string()).into(),
                (&json!({"b":2}), "$['a']['c']".to_string()).into(),
                (&json!({"b1":3}), "$['a']['d']".to_string()).into(),
            ])
        );
    }
}
