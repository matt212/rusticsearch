use rustc_serialize::json::Json;

use term::Term;

use query::{Query, TermMatcher};
use query::parser::{QueryParseContext, QueryParseError};
use query::parser::utils::parse_float;


pub fn parse(context: &QueryParseContext, json: &Json) -> Result<Query, QueryParseError> {
    let object = try!(json.as_object().ok_or(QueryParseError::ExpectedObject));

    // Prefix queries are very similar to term queries except that they will also match prefixes
    // of terms
    //
    // {
    //     "foo": "bar"
    // }
    //
    // {
    //     "foo": {
    //         "query": "bar",
    //         "boost": 2.0
    //     }
    // }
    //
    let field_name = if object.len() == 1 {
        object.keys().collect::<Vec<_>>()[0]
    } else {
        return Err(QueryParseError::ExpectedSingleKey)
    };

    let object = object.get(field_name).unwrap();

    // Get configuration
    let mut value: Option<&Json> = None;
    let mut boost = 1.0f64;

    match *object {
        Json::String(ref string) => value = Some(object),
        Json::Object(ref inner_object) => {
            for (key, val) in inner_object.iter() {
                match key.as_ref() {
                    "value" => {
                        value = Some(val);
                    }
                    "prefix" => {
                        value = Some(val);
                    }
                    "boost" => {
                        boost = try!(parse_float(val));
                    }
                    _ => return Err(QueryParseError::UnrecognisedKey(key.clone()))
                }
            }
        }
        _ => return Err(QueryParseError::ExpectedObjectOrString),
    }

    match value {
        Some(value) => {
            let mut query = Query::MatchTerm {
                field: field_name.clone(),
                term: Term::from_json(value),
                matcher: TermMatcher::Prefix,
            };

            // Add boost
            if boost != 1.0f64 {
                query = Query::BoostScore {
                    query: Box::new(query),
                    boost: boost,
                };
            }

            Ok(query)
        }
        None => Err(QueryParseError::ExpectedKey("value"))
    }
}


#[cfg(test)]
mod tests {
    use rustc_serialize::json::Json;

    use term::Term;
    use query::{Query, TermMatcher};
    use query::parser::{QueryParseContext, QueryParseError};
    use index::Index;

    use super::parse;

    #[test]
    fn test_prefix_query() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"value\": \"bar\"
            }
        }
        ").unwrap());

        assert_eq!(query, Ok(Query::MatchTerm {
            field: "foo".to_string(),
            term: Term::String("bar".to_string()),
            matcher: TermMatcher::Prefix
        }));
    }

    #[test]
    fn test_simple_prefix_query() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": \"bar\"
        }
        ").unwrap());

        assert_eq!(query, Ok(Query::MatchTerm {
            field: "foo".to_string(),
            term: Term::String("bar".to_string()),
            matcher: TermMatcher::Prefix
        }));
    }

    #[test]
    fn test_with_prefix_key() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"prefix\": \"bar\"
            }
        }
        ").unwrap());

        assert_eq!(query, Ok(Query::MatchTerm {
            field: "foo".to_string(),
            term: Term::String("bar".to_string()),
            matcher: TermMatcher::Prefix
        }));
    }

    #[test]
    fn test_with_boost() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"value\": \"bar\",
                \"boost\": 2.0
            }
        }
        ").unwrap());

        assert_eq!(query, Ok(Query::BoostScore {
            query: Box::new(Query::MatchTerm {
                field: "foo".to_string(),
                term: Term::String("bar".to_string()),
                matcher: TermMatcher::Prefix
            }),
            boost: 2.0f64,
        }));
    }

    #[test]
    fn test_with_boost_integer() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"value\": \"bar\",
                \"boost\": 2
            }
        }
        ").unwrap());

        assert_eq!(query, Ok(Query::BoostScore {
            query: Box::new(Query::MatchTerm {
                field: "foo".to_string(),
                term: Term::String("bar".to_string()),
                matcher: TermMatcher::Prefix
            }),
            boost: 2.0f64,
        }));
    }

    #[test]
    fn test_gives_error_for_incorrect_type() {
        // Array
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        [
            \"foo\"
        ]
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedObject));

        // Integer
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        123
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedObject));

        // Float
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        123.1234
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedObject));
    }

    #[test]
    fn test_gives_error_for_incorrect_boost_type() {
        // String
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"query\": \"bar\",
                \"boost\": \"2\"
            }
        }
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedFloat));

        // Array
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"query\": \"bar\",
                \"boost\": [2]
            }
        }
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedFloat));

        // Object
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"query\": \"bar\",
                \"boost\": {
                    \"value\": 2
                }
            }
        }
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedFloat));
    }

    #[test]
    fn test_gives_error_for_missing_value() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
            }
        }
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedKey("value")));
    }

    #[test]
    fn test_gives_error_for_extra_key() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"query\": \"bar\"
            },
            \"hello\": \"world\"
        }
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::ExpectedSingleKey));
    }

    #[test]
    fn test_gives_error_for_extra_inner_key() {
        let query = parse(&QueryParseContext::new(&Index::new()), &Json::from_str("
        {
            \"foo\": {
                \"query\": \"bar\",
                \"hello\": \"world\"
            }
        }
        ").unwrap());

        assert_eq!(query, Err(QueryParseError::UnrecognisedKey("hello".to_string())));
    }
}
