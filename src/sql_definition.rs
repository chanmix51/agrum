pub trait SqlDefinition {
    fn expand(&self, condition: &str) -> String;
}
