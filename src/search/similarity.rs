#[derive(Debug, PartialEq)]
pub enum SimilarityModel {
    TF_IDF,
    BM25{k1: f64, b: f64},
}


/// tf(term_frequency) = log(term_frequency) + 1.0
#[inline]
fn tf(term_frequency: u32) -> f64 {
    (term_frequency as f64).log(10.0) + 1.0
}


/// idf(term_docs, total_docs) = log((total_docs + 1.0) / (term_docs + 1.0)) + 1.0
#[inline]
fn idf(term_docs: u64, total_docs: u64) -> f64 {
    ((total_docs as f64 + 1.0) / (term_docs as f64 + 1.0)).log(10.0) + 1.0
}


impl SimilarityModel {
    pub fn score(&self, term_frequency: u32, length: u32, total_tokens: u64, total_docs: u64, total_docs_with_term: u64) -> f64 {
        match *self {
            SimilarityModel::TF_IDF => {
                let tf = tf(term_frequency);
                let idf = idf(total_docs_with_term, total_docs);

                tf * idf
            }
            SimilarityModel::BM25{k1, b} => {
                let tf = tf(term_frequency);
                let idf = idf(total_docs_with_term, total_docs);
                let average_length = (total_tokens as f64) / (total_docs as f64);

                idf * (k1 + 1.0) * (tf / (tf + (k1 * ((1.0 - b) + b * (length as f64).sqrt() / average_length.sqrt()))))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::SimilarityModel;

    #[test]
    fn test_tf_idf_higher_term_freq_increases_score() {
        let similarity = SimilarityModel::TF_IDF;

        assert!(similarity.score(2, 40, 100, 10, 5) > similarity.score(1, 40, 100, 10, 5));
    }

    #[test]
    fn test_tf_idf_lower_term_docs_increases_score() {
        let similarity = SimilarityModel::TF_IDF;

        assert!(similarity.score(1, 40, 100, 10, 5) > similarity.score(1, 40, 100, 10, 10));
    }

    #[test]
    fn test_tf_idf_field_length_doesnt_affect_score() {
        let similarity = SimilarityModel::TF_IDF;

        assert!(similarity.score(1, 100, 100, 20, 5) == similarity.score(1, 40, 100, 20, 5));
    }

    #[test]
    fn test_tf_idf_total_tokens_doesnt_affect_score() {
        let similarity = SimilarityModel::TF_IDF;

        assert!(similarity.score(1, 40, 1000, 20, 5) == similarity.score(1, 40, 100, 20, 5));
    }

    #[test]
    fn test_bm25_higher_term_freq_increases_score() {
        let similarity = SimilarityModel::BM25 {
            k1: 1.2,
            b: 0.75,
        };

        assert!(similarity.score(2, 40, 100, 10, 5) > similarity.score(1, 40, 100, 10, 5));
    }

    #[test]
    fn test_bm25_lower_term_docs_increases_score() {
        let similarity = SimilarityModel::BM25 {
            k1: 1.2,
            b: 0.75,
        };

        assert!(similarity.score(1, 40, 100, 10, 5) > similarity.score(1, 40, 100, 10, 10));
    }

    #[test]
    fn test_bm25_lower_field_length_increases_score() {
        let similarity = SimilarityModel::BM25 {
            k1: 1.2,
            b: 0.75,
        };

        assert!(similarity.score(1, 40, 100, 20, 5) > similarity.score(1, 100, 100, 20, 5));
    }

    #[test]
    fn test_bm25_higher_total_tokens_increases_score() {
        let similarity = SimilarityModel::BM25 {
            k1: 1.2,
            b: 0.75,
        };

        assert!(similarity.score(1, 40, 1000, 20, 5) > similarity.score(1, 40, 100, 20, 5));
    }
}
