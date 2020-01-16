use failure::_core::cmp::Ordering;
use handlebars::{
    BlockParams, Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext,
    RenderError, Renderable,
};
use serde_json::{Map, Value};

use log::debug;

use handlebars::to_json;
use serde_json::value::Value as Json;
use std::collections::BTreeMap;

pub(crate) fn copy_on_push_vec<T>(input: &[T], el: T) -> Vec<T>
where
    T: Clone,
{
    let mut new_vec = Vec::with_capacity(input.len() + 1);
    new_vec.extend_from_slice(input);
    new_vec.push(el);
    new_vec
}

pub trait JsonTruthy {
    fn is_truthy(&self, include_zero: bool) -> bool;
}

impl JsonTruthy for Json {
    fn is_truthy(&self, include_zero: bool) -> bool {
        match *self {
            Json::Bool(ref i) => *i,
            Json::Number(ref n) => {
                if include_zero {
                    n.as_f64().map(|f| !f.is_nan()).unwrap_or(false)
                } else {
                    // there is no inifity in json/serde_json
                    n.as_f64().map(|f| f.is_normal()).unwrap_or(false)
                }
            }
            Json::Null => false,
            Json::String(ref i) => !i.is_empty(),
            Json::Array(ref i) => !i.is_empty(),
            Json::Object(ref i) => !i.is_empty(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SortedHelper;

impl HelperDef for SortedHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let value = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"sort\""))?;

        let sort_path = h
            .param(1)
            .and_then(|v| v.value().as_str())
            .ok_or(RenderError::new("Missing parameter 2 (path) for sort"))?;

        let invert = h
            .hash_get("invert")
            .and_then(|v| v.value().as_bool())
            .unwrap_or(false);

        let template = h.template();

        match template {
            Some(t) => {
                let saved_path = rc.get_path().to_vec();
                rc.promote_local_vars();
                let local_path_root = value.path_root();
                if let Some(p) = local_path_root {
                    rc.push_local_path_root(p.to_vec());
                }

                debug!("each value {:?}", value.value());
                let rendered = match (value.value().is_truthy(false), value.value()) {
                    (true, &Json::Array(ref list)) => {
                        let len = list.len();

                        let array_path = value.context_path();
                        let sorted = sorted_array(list, sort_path, invert);

                        for (x, _) in list.iter().enumerate().take(len) {
                            let i = sorted[x];
                            rc.set_local_var("@first".to_string(), to_json(i == 0usize));
                            rc.set_local_var("@last".to_string(), to_json(i == len - 1));
                            rc.set_local_var("@index".to_string(), to_json(i));

                            if let Some(ref p) = array_path {
                                rc.set_path(copy_on_push_vec(p, i.to_string()))
                            }

                            if let Some(bp_val) = h.block_param() {
                                let mut params = BlockParams::new();
                                params.add_path(bp_val, rc.get_path().clone())?;

                                rc.push_block_context(params)?;
                            } else if let Some((bp_val, bp_index)) = h.block_param_pair() {
                                let mut params = BlockParams::new();
                                params.add_path(bp_val, rc.get_path().clone())?;
                                params.add_value(bp_index, to_json(i))?;

                                rc.push_block_context(params)?;
                            }

                            t.render(r, ctx, rc, out)?;

                            if h.has_block_param() {
                                rc.pop_block_context();
                            }
                        }

                        Ok(())
                    }
                    (true, &Json::Object(ref obj)) => {
                        let mut first: bool = true;
                        let obj_path = value.context_path();
                        let sorted: Vec<&String> = sorted_map(obj, sort_path, invert);

                        for k in sorted {
                            rc.set_local_var("@first".to_string(), to_json(first));
                            if first {
                                first = false;
                            }

                            rc.set_local_var("@key".to_string(), to_json(k));

                            if let Some(ref p) = obj_path {
                                rc.set_path(copy_on_push_vec(p, k.clone()));
                            }

                            if let Some(bp_val) = h.block_param() {
                                let mut params = BlockParams::new();
                                params.add_path(bp_val, rc.get_path().clone())?;

                                rc.push_block_context(params)?;
                            } else if let Some((bp_val, bp_key)) = h.block_param_pair() {
                                let mut params = BlockParams::new();
                                params.add_path(bp_val, rc.get_path().clone())?;
                                params.add_value(bp_key, to_json(&k))?;

                                rc.push_block_context(params)?;
                            }

                            t.render(r, ctx, rc, out)?;

                            if h.has_block_param() {
                                rc.pop_block_context();
                            }
                        }
                        Ok(())
                    }
                    (false, _) => {
                        if let Some(else_template) = h.inverse() {
                            else_template.render(r, ctx, rc, out)?;
                        }
                        Ok(())
                    }
                    _ => Err(RenderError::new(format!(
                        "Param type is not iterable: {:?}",
                        value.value()
                    ))),
                };

                if local_path_root.is_some() {
                    rc.pop_local_path_root();
                }

                rc.demote_local_vars();
                rc.set_path(saved_path);
                rendered
            }
            None => Ok(()),
        }
    }
}

fn sorted_array(list: &Vec<Value>, sort_path: &str, invert: bool) -> Vec<usize> {
    struct Entry<'a> {
        i: usize,
        value: &'a Value,
    }

    let len = list.len();
    let mut e: Vec<Entry> = Vec::with_capacity(len);

    for (i, _) in list.iter().enumerate().take(len) {
        e.push(Entry { i, value: &list[i] });
    }

    if invert {
        e.sort_by(|v1, v2| sort(sort_path, v2.value, v1.value));
    } else {
        e.sort_by(|v1, v2| sort(sort_path, v1.value, v2.value));
    }

    e.iter().map(|i| i.i).collect()
}

fn sorted_map<'a>(map: &'a Map<String, Value>, sort_path: &str, invert: bool) -> Vec<&'a String> {
    struct Entry<'a> {
        key: &'a String,
        value: &'a Value,
    }

    let len = map.len();
    let mut e: Vec<Entry> = Vec::with_capacity(len);

    for (k, v) in map.iter() {
        e.push(Entry { key: k, value: v });
    }

    if invert {
        e.sort_by(|v1, v2| sort(sort_path, v2.value, v1.value));
    } else {
        e.sort_by(|v1, v2| sort(sort_path, v1.value, v2.value));
    }

    e.iter().map(|i| i.key).collect()
}

fn sort(sort_path: &str, v1: &Value, v2: &Value) -> Ordering {
    let v1 = v1.pointer(sort_path).and_then(|v| v.as_str());
    let v2 = v2.pointer(sort_path).and_then(|v| v.as_str());

    let result = match (v1, v2) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(s1), Some(s2)) => s1.cmp(s2),
    };

    debug!("cmp - v1: {:?}, v2: {:?} => {:?}", v1, v2, result);

    result
}

#[cfg(test)]
mod test {
    use super::*;
    use log::{Level, LevelFilter};
    use serde::{Deserialize, Serialize};
    use serde_json::Map;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct Item {
        field1: String,
        field2: String,
    }

    fn init() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn test_sort_array_1() {
        init();

        let items = vec![
            Item {
                field1: "C".to_string(),
                field2: "C1".to_string(),
            },
            Item {
                field1: "A".to_string(),
                field2: "A1".to_string(),
            },
            Item {
                field1: "B".to_string(),
                field2: "B1".to_string(),
            },
        ];
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("sorted", Box::new(SortedHelper));
        assert!(handlebars
            .register_template_string(
                "t0",
                "{{#sorted items \"/field1\" as |sorted|}}{{this.field2}}{{/sorted}}"
            )
            .is_ok());
        assert!(handlebars
            .register_template_string(
                "t1",
                "{{#sorted items \"/field1\" invert=true as |sorted|}}{{this.field2}}{{/sorted}}"
            )
            .is_ok());

        let mut data = Map::new();
        data.insert("items".into(), serde_json::to_value(items).expect(""));
        let data = Value::Object(data);

        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.expect(""), "A1B1C1".to_string());
        let r1 = handlebars.render("t1", &data);
        assert_eq!(r1.expect(""), "C1B1A1".to_string());
    }

    #[test]
    fn test_sort_map_1() {
        init();

        let mut items = BTreeMap::new();
        items.insert(
            "key1",
            Item {
                field1: "C".to_string(),
                field2: "C1".to_string(),
            },
        );
        items.insert(
            "key2",
            Item {
                field1: "A".to_string(),
                field2: "A1".to_string(),
            },
        );
        items.insert(
            "key3",
            Item {
                field1: "B".to_string(),
                field2: "B1".to_string(),
            },
        );

        let items = serde_json::to_value(&items).expect("");

        let mut handlebars = Handlebars::new();
        handlebars.register_helper("sorted", Box::new(SortedHelper));
        assert!(handlebars
            .register_template_string(
                "t0",
                "{{#sorted items \"/field1\" as |sorted|}}{{this.field2}}{{/sorted}}"
            )
            .is_ok());
        assert!(handlebars
            .register_template_string(
                "t1",
                "{{#sorted items \"/field1\" invert=true as |sorted|}}{{this.field2}}{{/sorted}}"
            )
            .is_ok());

        let mut data = Map::new();
        data.insert("items".into(), serde_json::to_value(items).expect(""));
        let data = Value::Object(data);

        let r0 = handlebars.render("t0", &data);
        assert_eq!(r0.expect(""), "A1B1C1".to_string());
        let r1 = handlebars.render("t1", &data);
        assert_eq!(r1.expect(""), "C1B1A1".to_string());
    }
}
