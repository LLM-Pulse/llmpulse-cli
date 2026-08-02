use serde_json::Value;

pub type Query = Vec<(String, String)>;

pub trait QueryExt {
    fn push_opt<T: ToString>(&mut self, key: &str, value: Option<T>);
    fn push_bool(&mut self, key: &str, value: bool);
}

impl QueryExt for Query {
    fn push_opt<T: ToString>(&mut self, key: &str, value: Option<T>) {
        if let Some(value) = value {
            let value = value.to_string();
            if !value.is_empty() {
                self.push((key.to_owned(), value));
            }
        }
    }

    fn push_bool(&mut self, key: &str, value: bool) {
        if value {
            self.push((key.to_owned(), "true".to_owned()));
        }
    }
}

pub fn compact_object(mut value: Value) -> Value {
    if let Value::Object(object) = &mut value {
        object.retain(|_, value| !value.is_null());
    }
    value
}
