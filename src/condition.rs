use tokio_postgres::types::ToSql;

pub trait Condition {
    fn to_string(&self) -> String;
}

pub struct BooleanExpression {
    expression: String,
    parameters: Vec<Box<dyn ToSql + Sync>>,
}

impl BooleanExpression {
    pub fn new(expression: &str, parameters: Vec<Box<dyn ToSql + Sync>>) -> Self {
        Self {
            expression: expression.to_owned(),
            parameters,
        }
    }
}

impl Condition for BooleanExpression {
    fn to_string(&self) -> String {
        self.expression.clone()
    }
}

enum BooleanCondition {
    None,
    And(Box<dyn Condition>, Box<dyn Condition>),
    Or(Box<dyn Condition>, Box<dyn Condition>),
    Expression(BooleanExpression),
}

impl Condition for BooleanCondition {
    fn to_string(&self) -> String {
        match self {
            Self::None => "true".to_string(),
            Self::And(left, right) => format!("{} and {}", left.to_string(), right.to_string()),
            Self::Or(left, right) => format!("{} or {}", left.to_string(), right.to_string()),
            Self::Expression(expr) => expr.to_string(),
        }
    }
}

pub struct WhereCondition {
    condition: BooleanCondition,
}

impl WhereCondition {
    pub fn new(expression: &str, parameters: Vec<Box<dyn ToSql + Sync>>) -> Self {
        Self {
            condition: BooleanCondition::Expression(BooleanExpression::new(expression, parameters)),
        }
    }

    pub fn and_where(&mut self, condition: WhereCondition) -> &mut Self {
        let my_condition = self.condition;

        self.condition = BooleanCondition::And(Box::new(my_condition), Box::new(condition));

        self
    }
}

impl Default for WhereCondition {
    fn default() -> Self {
        Self {
            condition: BooleanCondition::None,
        }
    }
}

impl Condition for WhereCondition {
    fn to_string(&self) -> String {
        self.condition.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn where_condition_new() {
        let condition = WhereCondition::default();

        assert_eq!("true", &condition.to_string());
    }

    #[test]
    fn where_condition_expression() {
        let expression = "nothing is NULL";
        let condition = WhereCondition::new(expression, Vec::new());

        assert_eq!(expression, &condition.to_string());
    }

    #[test]
    fn where_and() {}
}
