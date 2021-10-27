use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    str::Lines,
};

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

#[derive(Debug)]
pub struct HeaderMap(pub HashMap<HeaderKey, String>);

impl HeaderMap {
    pub fn from_lines(lines: Lines) -> Self {
        let headers = lines.fold(HashMap::new(), |mut curr, next| {
            let header = next.split_once(':');

            if let Some((k, v)) = header {
                curr.insert(HeaderKey(k.trim().to_owned()), v.trim().to_owned());
            }

            curr
        });

        HeaderMap(headers)
    }

    pub fn get_by_key(&self, key: &str) -> Option<&str> {
        let key = HeaderKey(key.to_owned());

        self.0.get(&key).map(|k| k.as_str())
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
