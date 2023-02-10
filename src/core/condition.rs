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
            Self::And(lft, rgt) => match (lft.needs_precedence(), rgt.needs_precedence()) {
                (true, false) => format!("({}) and {}", lft.expand(), rgt.expand()),
                (false, true) => format!("{} and ({})", lft.expand(), rgt.expand()),
                (true, true) => format!("({}) and ({})", lft.expand(), rgt.expand()),
                (false, false) => format!("{} and {}", lft.expand(), rgt.expand()),
            },
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

pub struct WhereCondition<'a> {
    condition: BooleanCondition,
    parameters: Vec<&'a (dyn ToSql + Sync)>,
}

impl<'a> Default for WhereCondition<'a> {
    fn default() -> Self {
        Self {
            condition: BooleanCondition::None,
            parameters: Vec::new(),
        }
    }
}

impl<'a> WhereCondition<'a> {
    pub fn new(expression: &str, parameters: Vec<&'a (dyn ToSql + Sync)>) -> Self {
        Self {
            condition: BooleanCondition::Expression(expression.to_string()),
            parameters,
        }
    }

    pub fn expand(self) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        let mut expression = self.condition.expand();
        let parameters = self.parameters;
        let mut param_index = 1;
        //
        // Replace parameters placeholders by numerated parameters.
        loop {
            if !expression.contains("$?") {
                break;
            }
            expression = expression.replacen("$?", &format!("${param_index}"), 1);
            param_index += 1;
        }

        (expression, parameters)
    }

    pub fn where_in(field: &str, parameters: Vec<&'a (dyn ToSql + Sync)>) -> Self {
        let params: Vec<&str> = repeat("$?").take(parameters.len()).collect();
        let expression = format!("{} in ({})", field, params.join(", "));

        Self {
            condition: BooleanCondition::Expression(expression),
            parameters,
        }
    }

    pub fn and_where(&mut self, mut condition: WhereCondition<'a>) -> &mut Self {
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

    pub fn or_where(&mut self, mut condition: WhereCondition<'a>) -> &mut Self {
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
        assert!(params.is_empty());
    }

    #[test]
    fn expression_sql() {
        let expression = WhereCondition::new("something is not null", Vec::new());
        let (sql, params) = expression.expand();

        assert_eq!("something is not null".to_string(), sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_and() {
        let mut expression = WhereCondition::new("A", Vec::new());
        expression.and_where(WhereCondition::new("B", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A and B", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_and_none() {
        let mut expression = WhereCondition::new("A", Vec::new());
        expression.and_where(WhereCondition::default());
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_none_and() {
        let mut expression = WhereCondition::default();
        expression.and_where(WhereCondition::new("A", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_or() {
        let mut expression = WhereCondition::new("A", Vec::new());
        expression.or_where(WhereCondition::new("B", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A or B", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_or_none() {
        let mut expression = WhereCondition::new("A", Vec::new());
        expression.or_where(WhereCondition::default());
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_none_or() {
        let mut expression = WhereCondition::default();
        expression.or_where(WhereCondition::new("A", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_no_precedence() {
        let mut expression = WhereCondition::new("A", Vec::new());
        expression
            .and_where(WhereCondition::new("B", Vec::new()))
            .or_where(WhereCondition::new("C", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A and B or C", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_with_precedence() {
        let mut sub_expression = WhereCondition::new("A", Vec::new());
        sub_expression.or_where(WhereCondition::new("B", Vec::new()));
        let mut expression = WhereCondition::new("C", Vec::new());
        expression.and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!("C and (A or B)", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_with_self_precedence() {
        let mut expression = WhereCondition::new("A", Vec::new());
        expression.or_where(WhereCondition::new("B", Vec::new()));
        let sub_expression = WhereCondition::new("C", Vec::new());
        expression.and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!("(A or B) and C", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_with_both_precedence() {
        let mut expression = WhereCondition::new("A", Vec::new());
        expression.or_where(WhereCondition::new("B", Vec::new()));
        let mut sub_expression = WhereCondition::new("C", Vec::new());
        sub_expression.or_where(WhereCondition::new("D", Vec::new()));
        expression.and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!("(A or B) and (C or D)", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_sql_with_parameter() {
        let expression = WhereCondition::new("A > $?::pg_type", vec![&(0_i32)]);
        let (sql, params) = expression.expand();

        assert_eq!("A > $1::pg_type", &sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_sql_with_multiple_parameters() {
        let mut expression = WhereCondition::new("A > $?::pg_type", vec![&(0_i32)]);
        expression.and_where(WhereCondition::new("B = $?", vec![&(1_i32)]));
        let (sql, params) = expression.expand();

        assert_eq!("A > $1::pg_type and B = $2", &sql);
        assert_eq!(2, params.len());
    }

    #[test]
    fn expression_where_in() {
        let expression = WhereCondition::where_in("A", vec![&(0_i32), &(1_i32)]);
        let (sql, params) = expression.expand();

        assert_eq!("A in ($1, $2)".to_string(), sql);
        assert_eq!(2, params.len());
    }

    #[test]
    fn expression_sql_with_multiple_parameters_and_where_in() {
        let mut expression = WhereCondition::new("A > $?::pg_type", vec![&(0_i32)]);
        expression
            .or_where(WhereCondition::new("B", Vec::new()))
            .and_where(WhereCondition::where_in(
                "C",
                vec![&100_i32, &101_i32, &102_i32],
            ));

        let (sql, params) = expression.expand();

        assert_eq!("(A > $1::pg_type or B) and C in ($2, $3, $4)", &sql);
        assert_eq!(4, params.len());
    }

    #[test]
    #[should_panic]
    fn expression_with_wrong_number_of_parameters_panics() {
        let expression = WhereCondition::new("A > $?::pg_type", Vec::new());
        let _ = expression.expand();
    }
}
