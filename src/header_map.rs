use std::{
    collections::HashMap,
    io::Write,
    ops::{Deref, DerefMut},
    str::{FromStr, Lines},
};

use crate::Result;

#[derive(Debug, Eq)]
pub struct HeaderKey(pub String);

impl std::hash::Hash for HeaderKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}

impl std::cmp::PartialEq for HeaderKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

#[derive(Debug, Default)]
pub struct HeaderMap(pub HashMap<HeaderKey, String>);

impl HeaderMap {
    pub fn from_lines(lines: Lines) -> Self {
        let headers = lines.fold(HashMap::new(), |mut curr, next| {
            let header = next.split_once(": ");

            if let Some((k, v)) = header {
                curr.insert(HeaderKey(k.trim().to_owned()), v.to_owned());
            }

            curr
        });

        HeaderMap(headers)
    }

    pub fn get_by_str_key(&self, key: &str) -> Option<&str> {
        let key = HeaderKey(key.to_owned());

        self.0.get(&key).map(|k| k.as_str())
    }

    pub fn get_by_str_key_as<T: FromStr>(&self, key: &str) -> Option<T> {
        let key = HeaderKey(key.to_owned());

        self.0.get(&key).and_then(|k| k.parse().ok())
    }

    pub fn insert_by_str_key_value(&mut self, key: &str, value: &str) {
        let key = HeaderKey(key.to_owned());

        self.insert(key, value.to_owned());
    }

    pub fn contains_by_str_key_value(&self, key: &str, value: &str) -> bool {
        let key = HeaderKey(key.to_owned());

        self.0
            .get(&key)
            .map(|v| v.eq_ignore_ascii_case(value))
            .unwrap_or(false)
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> Result<()> {
        for (k, v) in self.iter() {
            write!(writer, "{}: {}\r\n", k.0, v)?;
        }

        Ok(())
    }
}

impl Deref for HeaderMap {
    type Target = HashMap<HeaderKey, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HeaderMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
