use std::collections::{hash_map::Iter, HashMap};

use super::{SqlDefinition, Structure};

/// This represent a SQL data source. It can be a table, a SQL function, a query
/// etc. A Source is a definition hence it can be expanded.
pub trait SqlSource: SqlDefinition {
    /// Return the structure of the tuple provided by the source.
    fn get_structure(&self) -> Structure;
}

#[derive(Default)]
pub struct SourcesCatalog {
    sources: HashMap<String, Box<dyn SqlSource>>,
}

impl SourcesCatalog {
    pub fn new(sources: HashMap<String, Box<dyn SqlSource>>) -> Self {
        Self { sources }
    }

    pub fn add_source(&mut self, name: &str, source: Box<dyn SqlSource>) -> &mut Self {
        self.sources.insert(name.to_string(), source);

        self
    }

    /// Expand the given source's definition. Panics if the source does not exist.
    pub fn expand(&self, source: &str, condition: &str) -> String {
        self.sources
            .get(source)
            .unwrap_or_else(|| {
                panic!(
                    "Cannot expand unknown source '{source}'. Sources are [{}].",
                    self.sources
                        .keys()
                        .map(|k| k.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                )
            })
            .expand(condition)
    }

    pub fn iter(&self) -> Iter<'_, String, Box<dyn SqlSource>> {
        self.sources.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSource;

    impl SqlSource for TestSource {
        fn get_structure(&self) -> Structure {
            Structure::default()
        }
    }

    impl SqlDefinition for TestSource {
        fn expand(&self, condition: &str) -> String {
            format!("DEF COND[{condition}]")
        }
    }

    #[test]
    fn expand_source_catalog() {
        let mut catalog = SourcesCatalog::default();
        catalog.add_source("some_source", Box::new(TestSource));

        assert_eq!(
            "DEF COND[whatever]".to_string(),
            catalog.expand("some_source", "whatever")
        );
    }

    #[test]
    #[should_panic]
    fn expand_source_panics() {
        let mut catalog = SourcesCatalog::default();
        catalog.add_source("some_source", Box::new(TestSource));

        let _ = catalog.expand("bad_source", "");
    }
}
