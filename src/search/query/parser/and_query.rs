use rustc_serialize::json::Json;

use search::query::Query;
use search::query::parser::{QueryParseContext, QueryParseError, parse as parse_query};


pub fn parse(context: &QueryParseContext, json: &Json) -> Result<Query, QueryParseError> {
    let filters = try!(json.as_array().ok_or(QueryParseError::ExpectedArray));
    let mut sub_queries = Vec::new();

    for filter in filters.iter() {
        sub_queries.push(try!(parse_query(context, filter)));
    }

    Ok(Query::new_conjunction(sub_queries))
}


#[cfg(test)]
mod tests {
    use rustc_serialize::json::Json;

    use search::term::Term;
    use search::query::{Query, TermMatcher};
    use search::query::parser::{QueryParseContext, QueryParseError};

    use super::parse;

    #[test]
    fn test_and_query() {
        let query = parse(&QueryParseContext::new(), &Json::from_str("
        [
            {
                \"term\": {
                    \"test\":  \"foo\"
                }
            },
            {
                \"term\": {
                    \"test\":  \"bar\"
                }
            }
        ]
        ").unwrap());

        assert_eq!(query, Ok(Query::Conjunction {
            queries: vec![
                Query::MatchTerm {
                    field: "test".to_string(),
                    term: Term::String("foo".to_string()),
                    matcher: TermMatcher::Exact
                },
                Query::MatchTerm {
                    field: "test".to_string(),
                    term: Term::String("bar".to_string()),
                    matcher: TermMatcher::Exact
                },
            ],
        }))
    }

    #[test]
    fn test_gives_error_for_incorrect_type() {
        // String
        let query = parse(&QueryParseContext::new(), &Json::from_str("
        \"hello\"
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedArray));

        // Object
        let query = parse(&QueryParseContext::new(), &Json::from_str("
        {
            \"foo\": \"bar\"
        }
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedArray));

        // Integer
        let query = parse(&QueryParseContext::new(), &Json::from_str("
        123
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedArray));

        // Float
        let query = parse(&QueryParseContext::new(), &Json::from_str("
        123.1234
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedArray));
    }
}