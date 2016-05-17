use {Document, Value};

use query::Query;


impl Query {
    pub fn rank(&self, doc: &Document) -> Option<f64> {
        match *self {
            Query::MatchAll{boost} => Some(boost),
            Query::MatchNone => None,
            Query::MatchTerm{ref field, ref value, ref matcher, boost} => {
                if let Some(field_value) = doc.fields.get(field) {
                    match *field_value {
                        Value::String(ref field_value) => {
                            if let Value::String(ref value) = *value {
                                return if matcher.matches(field_value, value) { Some(boost) } else { None };
                            }
                        }
                        Value::TSVector(ref field_value) => {
                            if let Value::String(ref value) = *value {
                                let mut matched_terms = 0;
                                for field_term in field_value.iter() {
                                    if matcher.matches(field_term, value) {
                                        matched_terms += 1;
                                    }
                                }

                                if matched_terms > 0 {
                                    return Some(matched_terms as f64 * boost);
                                }
                            }
                        }
                        _ => return None
                    }
                }

                None
            }
            Query::Bool{ref must, ref must_not, ref should, ref filter, minimum_should_match, boost} => {
                let mut total_score: f64 = 0.0;

                // Must not
                for query in must_not {
                    if query.matches(doc) {
                        return None;
                    }
                }

                // Filter
                for filter in filter {
                    if !filter.matches(doc) {
                        return None;
                    }
                }

                // Must
                for query in must {
                    match query.rank(doc) {
                        Some(score) => {
                            total_score += score;
                        }
                        None => return None,
                    }
                }

                // Should
                let mut should_matched: i32 = 0;
                for query in should {
                    if let Some(score) = query.rank(doc) {
                        should_matched += 1;
                        total_score += score;
                    }
                }

                if should_matched < minimum_should_match {
                    return None;
                }

                // Return average score of matched queries
                Some((total_score * boost) / (must.len() + should.len()) as f64)
            }
            Query::DisjunctionMax{ref queries, boost} => {
                let mut something_matched = false;
                let mut max_score = 0.0f64;

                for query in queries {
                    match query.rank(doc) {
                        Some(score) => {
                            something_matched = true;
                            if score > max_score {
                                max_score = score;
                            }
                        }
                        None => continue,
                    }
                }

                if something_matched {
                    Some(max_score * boost)
                } else {
                    None
                }
            }
        }
    }
}