use std::{fmt::Display, iter::repeat_n};

use tokio_postgres::types::ToSql;

/// A trait to mark types that can be converted to a `ToSql` type and also
/// implement `Any` and `Sync`. This trait is used for the parameters of the
/// queries.
pub trait ToSqlAny: ToSql + std::any::Any + Sync {}
impl<T: ToSql + std::any::Any + Sync> ToSqlAny for T {}

/// A macro to create a vector of parameters. This macro is used to create the
/// parameters of the queries.
#[macro_export]
macro_rules! params {
    ($( $x:expr ),*) => {
        {
            let params = vec![$(&$x as &dyn $crate::ToSqlAny),*];
            params
        }
    };
}

/// A structure to hold the boolean conditions of the queries.
/// It is used to implement the precedence of the boolean logic operators.
#[derive(Debug, Clone)]
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

/// A structure to hold the where condition of the queries.
/// It is designed to be a composable structure to build the where condition of
/// the queries alongside the according parameters, preserving the order of the
/// parameters.
/// The conditions are composed using the `and_where` and `or_where` methods
/// with a default representation of `true` if no condition is provided.
///
/// # Examples
/// ## Default condition
/// The default condition is `true` which is useful when the conditions are
/// passed as parameters.  This way, when no condition is provided, the
/// query will be like `select * from table where true`. The condition will
/// be ignored by the database planner and will return all the records.
///
/// ```rust
/// use agrum::{WhereCondition, params};
///
/// let condition = WhereCondition::default();
/// assert_eq!("true", condition.to_string());
/// ```
/// ## Building a condition
/// The `WhereCondition` structure can be built using the `new` method to
/// create a new condition with a SQL expression and the parameters.
/// The `and_where` and `or_where` methods can be used to compose the conditions
/// with the boolean logic operators.
/// The `expand` method can be used to get the SQL expression and the parameters
/// (consuming the instance).
///
/// ```rust
/// use agrum::{WhereCondition, params};
///
/// let condition = WhereCondition::new("A = $?", params![1_i32]);
/// let condition = condition.and_where(WhereCondition::new("B = $?", params![2_i32]));
/// let condition = condition.or_where(WhereCondition::new("C = $?", params![3_i32]));
/// assert_eq!("A = $? and B = $? or C = $?", condition.to_string());
/// ```
#[derive(Debug, Clone)]
pub struct WhereCondition<'a> {
    condition: BooleanCondition,
    parameters: Vec<&'a dyn ToSqlAny>,
}

impl<'a> Default for WhereCondition<'a> {
    fn default() -> Self {
        Self {
            condition: BooleanCondition::None,
            parameters: Vec::new(),
        }
    }
}

impl Display for WhereCondition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.condition.expand())
    }
}

impl<'a> WhereCondition<'a> {
    /// Create a new condition with a SQL expression and the parameters.
    pub fn new(expression: &str, parameters: Vec<&'a dyn ToSqlAny>) -> Self {
        Self {
            condition: BooleanCondition::Expression(expression.to_string()),
            parameters,
        }
    }

    /// Expand the condition to a SQL expression and the parameters (consuming the instance).
    /// This is normally used to get the SQL expression and the parameters.
    pub fn expand(self) -> (String, Vec<&'a dyn ToSqlAny>) {
        let expression = self.condition.expand();
        let parameters = self.parameters;

        (expression, parameters)
    }

    /// Create a new condition with a `IN` SQL expression and the parameters.
    /// It creates as many `$?` placeholders as the number of parameters.
    pub fn where_in(field: &str, parameters: Vec<&'a dyn ToSqlAny>) -> Self {
        let params: Vec<&str> = repeat_n("$?", parameters.len()).collect();
        let expression = format!("{} in ({})", field, params.join(", "));

        Self {
            condition: BooleanCondition::Expression(expression),
            parameters,
        }
    }

    /// Compose the condition with a `AND` boolean logic operator.
    pub fn and_where(mut self, mut condition: WhereCondition<'a>) -> Self {
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

    /// Compose the condition with a `OR` boolean logic operator.
    pub fn or_where(mut self, mut condition: WhereCondition<'a>) -> Self {
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
        let expression =
            WhereCondition::new("A", Vec::new()).and_where(WhereCondition::new("B", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A and B", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_and_none() {
        let expression = WhereCondition::new("A", Vec::new()).and_where(WhereCondition::default());
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_none_and() {
        let expression = WhereCondition::default().and_where(WhereCondition::new("A", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_or() {
        let expression =
            WhereCondition::new("A", Vec::new()).or_where(WhereCondition::new("B", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A or B", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_or_none() {
        let expression = WhereCondition::new("A", Vec::new()).or_where(WhereCondition::default());
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_none_or() {
        let expression = WhereCondition::default().or_where(WhereCondition::new("A", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_no_precedence() {
        let expression = WhereCondition::new("A", Vec::new())
            .and_where(WhereCondition::new("B", Vec::new()))
            .or_where(WhereCondition::new("C", Vec::new()));
        let (sql, params) = expression.expand();

        assert_eq!("A and B or C", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_with_precedence() {
        let sub_expression =
            WhereCondition::new("A", Vec::new()).or_where(WhereCondition::new("B", Vec::new()));
        let expression = WhereCondition::new("C", Vec::new()).and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!("C and (A or B)", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_with_self_precedence() {
        let sub_expression = WhereCondition::new("C", Vec::new());
        let expression = WhereCondition::new("A", Vec::new())
            .or_where(WhereCondition::new("B", Vec::new()))
            .and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!("(A or B) and C", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_complex_with_both_precedence() {
        let sub_expression =
            WhereCondition::new("C", Vec::new()).or_where(WhereCondition::new("D", Vec::new()));
        let expression = WhereCondition::new("A", Vec::new())
            .or_where(WhereCondition::new("B", Vec::new()))
            .and_where(sub_expression);
        let (sql, params) = expression.expand();

        assert_eq!("(A or B) and (C or D)", &sql);
        assert!(params.is_empty());
    }

    #[test]
    fn expression_sql_with_parameter() {
        let expression = WhereCondition::new("A > $?::pg_type", params![0_i32]);
        let (sql, params) = expression.expand();

        assert_eq!("A > $?::pg_type", &sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn expression_sql_with_multiple_parameters() {
        let expression = WhereCondition::new("A > $?::pg_type", params![0_i32])
            .and_where(WhereCondition::new("B = $?", params![1_i32]));
        let (sql, params) = expression.expand();

        assert_eq!("A > $?::pg_type and B = $?", &sql);
        assert_eq!(2, params.len());
    }

    #[test]
    fn expression_where_in() {
        let expression = WhereCondition::where_in("A", params![0_i32, 1_i32]);
        let (sql, params) = expression.expand();

        assert_eq!("A in ($?, $?)".to_string(), sql);
        assert_eq!(2, params.len());
    }

    #[test]
    fn expression_sql_with_multiple_parameters_and_where_in() {
        let expression = WhereCondition::new("A > $?::pg_type", params![0_i32])
            .or_where(WhereCondition::new("B", Vec::new()))
            .and_where(WhereCondition::where_in(
                "C",
                params![100_i32, 101_i32, 102_i32],
            ));

        let (sql, params) = expression.expand();

        assert_eq!("(A > $?::pg_type or B) and C in ($?, $?, $?)", &sql);
        assert_eq!(4, params.len());
    }

    #[test]
    fn parameters_tosql() {
        let expression = WhereCondition::new("a = $?", params!["whatever"]);
        let (sql, params) = expression.expand();

        assert_eq!("a = $?", &sql);
        assert_eq!(1, params.len());
    }

    #[test]
    fn test_to_string() {
        let expression = WhereCondition::new("a = $?", params![1_i32]);
        let sql = expression.to_string();

        assert_eq!("a = $?", &sql);
    }
}
