use crate::WhereCondition;

pub trait SqlDefinition {
    fn expand(&self, condition: &WhereCondition) -> String;
}
