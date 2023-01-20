use std::iter::repeat;

use tokio_postgres::types::ToSql;

enum BooleanCondition {
    None,
    Expression(String),
    And(Box<BooleanCondition>, Box<BooleanCondition>),
    Or(Box<BooleanCondition>, Box<BooleanCondition>),
}

impl BooleanCondition {
    pub fn expand(&self) -> String {
        match self {
            Self::None => "true".to_string(),
            Self::Expression(expr) => expr.to_owned(),
            Self::And(lft, rgt) => {
                if lft.needs_precedence() && !rgt.needs_precedence() {
                    format!("({}) and {}", lft.expand(), rgt.expand())
                } else if !lft.needs_precedence() && rgt.needs_precedence() {
                    format!("{} and ({})", lft.expand(), rgt.expand())
                } else if lft.needs_precedence() && rgt.needs_precedence() {
                    format!("({}) and ({})", lft.expand(), rgt.expand())
                } else {
                    format!("{} and {}", lft.expand(), rgt.expand())
                }
            }
            Self::Or(lft, rgt) => format!("{} or {}", lft.expand(), rgt.expand()),
        }
    }

    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn needs_precedence(&self) -> bool {
        matches!(self, Self::Or(_, _))
    }
}

pub struct WhereCondition {
    condition: BooleanCondition,
    parameters: Vec<Box<dyn ToSql + Sync>>,
}

impl Default for WhereCondition {
    fn default() -> Self {
        Self {
            condition: BooleanCondition::None,
            parameters: Vec::new(),
        }
    }
}

impl WhereCondition {
    pub fn new(expression: &str, parameters: Vec<Box<dyn ToSql + Sync>>) -> Self {
        Self {
            condition: BooleanCondition::Expression(expression.to_string()),
            parameters,
        }
    }

    pub fn expand(self) -> (String, Vec<Box<dyn ToSql + Sync>>) {
        let expression = self.condition.expand();
        let parameters = self.parameters;

        (expression, parameters)
    }

    pub fn where_in(field: &str, parameters: Vec<Box<dyn ToSql + Sync>>) -> Self {
        let params: Vec<&str> = repeat("?").take(parameters.len()).collect();
        let expression = format!("{} in ({})", field, params.join(", "));

        Self {
            condition: BooleanCondition::Expression(expression),
            parameters,
        }
    }

    pub fn and_where(&mut self, mut condition: WhereCondition) -> &mut Self {
        if condition.condition.is_none() {
            return self;
        }
        if self.condition.is_none() {
            self.condition = condition.condition;
            self.parameters = condition.parameters;
        } else {
            let temp = BooleanCondition::None;
            let my_condition = std::mem::replace(&mut self.condition, temp);
            self.condition =
                BooleanCondition::And(Box::new(my_condition), Box::new(condition.condition));
            self.parameters.append(&mut condition.parameters);
        }

        self
    }

    pub fn or_where(&mut self, mut condition: WhereCondition) -> &mut Self {
        if condition.condition.is_none() {
            return self;
        }
        if self.condition.is_none() {
            self.condition = condition.condition;
            self.parameters = condition.parameters;
        } else {
            let temp = BooleanCondition::None;
            let my_condition = std::mem::replace(&mut self.condition, temp);
            self.condition =
                BooleanCondition::Or(Box::new(my_condition), Box::new(condition.condition));
            self.parameters.append(&mut condition.parameters);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_expand_none() {
        let condition = BooleanCondition::None;

        assert_eq!("true".to_string(), condition.expand());
    }

    #[test]
    fn boolean_expand_expression() {
        let condition = BooleanCondition::Expression("something".to_string());

        assert_eq!("something".to_string(), condition.expand());
    }

    #[test]
    fn boolean_expand_and() {
        let left = BooleanCondition::Expression("left".to_string());
        let right = BooleanCondition::Expression("right".to_string());
        let condition = BooleanCondition::And(Box::new(left), Box::new(right));

        assert_eq!("left and right".to_string(), condition.expand());
    }

    #[test]
    fn boolean_expand_or() {
        let left = BooleanCondition::Expression("left".to_string());
        let right = BooleanCondition::Expression("right".to_string());
        let condition = BooleanCondition::Or(Box::new(left), Box::new(right));

        assert_eq!("left or right".to_string(), condition.expand());
    }

    #[test]
    fn expression_default() {
        let expression = WhereCondition::default();
        let (sql, params) = expression.expand();

        assert_eq!("true".to_string(), sql);
        assert_eq!(0, params.len());
    }

    #[test]
    fn expression_sql() {
        let expression = WhereCondition::new("something is not null", Vec::new());
        let (sql, params) = expression.expand();

        assert_eq!("something is not null".to_string(), sql);
        assert_eq!(0, params.len());
    }

    #[test]
    fn expression_sql_and_parameters() {
        let expression = WhereCondition::new("balance > ?", vec![Box::new(0 as u32)]);
        let (sql, params) = expression.expand();

        assert_eq!("balance > ?".to_string(), sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_where_in() {
        let expression = WhereCondition::where_in("something", vec![Box::new(1), Box::new(2)]);
        let (sql, params) = expression.expand();

        assert_eq!("something in (?, ?)".to_string(), sql);
        assert_eq!(2, params.len());
    }

    #[test]
    fn expression_and() {
        let mut expression = WhereCondition::new("something is not null", Vec::new());
        expression.and_where(WhereCondition::new("balance > ?", vec![Box::new(0)]));
        let (sql, params) = expression.expand();

        assert_eq!("something is not null and balance > ?".to_string(), sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_and_none() {
        let mut expression = WhereCondition::new("something is not null", Vec::new());
        expression.and_where(WhereCondition::default());
        let (sql, params) = expression.expand();

        assert_eq!("something is not null".to_string(), sql);
        assert_eq!(0, params.len());
    }

    #[test]
    fn expression_none_and() {
        let mut expression = WhereCondition::default();
        expression.and_where(WhereCondition::new("balance > ?", vec![Box::new(0)]));
        let (sql, params) = expression.expand();

        assert_eq!("balance > ?".to_string(), sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_or() {
        let mut expression = WhereCondition::new("something is not null", Vec::new());
        expression.or_where(WhereCondition::new("balance > ?", vec![Box::new(0)]));
        let (sql, params) = expression.expand();

        assert_eq!("something is not null or balance > ?".to_string(), sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_or_none() {
        let mut expression = WhereCondition::new("something is not null", Vec::new());
        expression.or_where(WhereCondition::default());
        let (sql, params) = expression.expand();

        assert_eq!("something is not null".to_string(), sql);
        assert_eq!(0, params.len());
    }

    #[test]
    fn expression_none_or() {
        let mut expression = WhereCondition::default();
        expression.or_where(WhereCondition::new("balance > ?", vec![Box::new(0)]));
        let (sql, params) = expression.expand();

        assert_eq!("balance > ?".to_string(), sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_complex_no_precedence() {
        let mut expression = WhereCondition::new("something is not null", Vec::new());
        expression
            .and_where(WhereCondition::new("balance > ?", vec![Box::new(0)]))
            .or_where(WhereCondition::new("has_superpower", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!(
            "something is not null and balance > ? or has_superpower".to_string(),
            sql
        );
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_complex_with_precedence() {
        let mut sub_expression = WhereCondition::new("balance > ?", vec![Box::new(0)]);
        sub_expression.or_where(WhereCondition::new("has_superpower", Vec::new()));
        let mut expression = WhereCondition::new("something is not null", Vec::new());
        expression.and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!(
            "something is not null and (balance > ? or has_superpower)".to_string(),
            sql
        );
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_complex_with_self_precedence() {
        let mut expression = WhereCondition::new("balance > ?", vec![Box::new(0)]);
        expression.or_where(WhereCondition::new("has_superpower", Vec::new()));
        let sub_expression = WhereCondition::new("something is not null", Vec::new());
        expression.and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!(
            "(balance > ? or has_superpower) and something is not null".to_string(),
            sql
        );
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_complex_with_both_precedence() {
        let mut expression = WhereCondition::new("A > ?", vec![Box::new(0)]);
        expression.or_where(WhereCondition::new("B", Vec::new()));
        let mut sub_expression = WhereCondition::new("C", Vec::new());
        sub_expression.or_where(WhereCondition::where_in(
            "D",
            vec![Box::new(10), Box::new(11)],
        ));
        expression.and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!("(A > ? or B) and (C or D in (?, ?))".to_string(), sql);
        assert_eq!(3, params.len());
    }
}
