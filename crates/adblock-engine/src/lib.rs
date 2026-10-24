use adblock::{Engine, lists::FilterSet, lists::ParseOptions, request::Request};
use thiserror::Error;

pub const FILTER_RULES: &str = include_str!("../../../assets/filters/spotify-ads.txt");

#[derive(Debug, Error)]
pub enum BlockError {
    #[error("empty filter list")]
    EmptyRules,
    #[error("invalid request")]
    InvalidRequest,
}

pub struct Blocker {
    engine: Engine,
}

impl Blocker {
    pub fn from_rules(rules: &str) -> Result<Self, BlockError> {
        if rules.trim().is_empty() {
            return Err(BlockError::EmptyRules);
        }
        let mut set = FilterSet::new(false);
        set.add_filter_list(rules.to_owned(), ParseOptions::default());
        Ok(Self {
            engine: Engine::new_with_filter_set(set),
        })
    }

    pub fn from_embedded() -> Result<Self, BlockError> {
        Self::from_rules(FILTER_RULES)
    }

    pub fn check(
        &self,
        url: &str,
        source_url: &str,
        request_type: &str,
    ) -> Result<bool, BlockError> {
        let request = Request::new(url, source_url, request_type, "get")
            .map_err(|_| BlockError::InvalidRequest)?;
        Ok(self.engine.check_network_request(&request).should_block())
    }

    pub fn is_blocked(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        self.check(url, source_url, request_type).unwrap_or(false)
    }
}

pub fn patterns_from_rules(rules: &str) -> Vec<String> {
    let mut patterns: Vec<String> = rules
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("||"))
        .filter_map(|line| {
            let rest = &line[2..];
            let end = rest
                .find(&['^', '$', '*', '?', '#', ','][..])
                .unwrap_or(rest.len());
            let mut pattern = rest[..end].trim().to_lowercase();
            while pattern.ends_with('/') {
                pattern.pop();
            }
            let keep = pattern.contains('.')
                && !pattern.contains(char::is_whitespace)
                && !pattern.contains('|');
            keep.then_some(pattern)
        })
        .collect();
    patterns.sort();
    patterns.dedup();
    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = "||ads.example.com^\n||spclient.wg.spotify.com/ads^\n@@||ads.example.com/allow^\n! comment\n";

    #[test]
    fn empty_rules_rejected() {
        assert!(Blocker::from_rules("   \n").is_err());
    }

    #[test]
    fn ad_request_blocked() {
        let blocker = Blocker::from_rules(RULES).unwrap();
        assert!(blocker.is_blocked(
            "https://ads.example.com/banner.js",
            "https://open.spotify.com/",
            "script"
        ));
        assert!(blocker.is_blocked(
            "https://spclient.wg.spotify.com/ads/v1/audio",
            "https://open.spotify.com/",
            "xmlhttprequest"
        ));
    }

    #[test]
    fn legitimate_traffic_allowed() {
        let blocker = Blocker::from_rules(RULES).unwrap();
        assert!(!blocker.is_blocked(
            "https://open.spotify.com/track/abc",
            "https://open.spotify.com/",
            "document"
        ));
        assert!(!blocker.is_blocked(
            "https://spclient.wg.spotify.com/metadata/abc",
            "https://open.spotify.com/",
            "xmlhttprequest"
        ));
    }

    #[test]
    fn invalid_request_allowed() {
        let blocker = Blocker::from_rules(RULES).unwrap();
        assert!(!blocker.is_blocked("://[invalide", "https://open.spotify.com/", "script"));
        assert!(
            blocker
                .check("://[invalide", "https://open.spotify.com/", "script")
                .is_err()
        );
    }

    #[test]
    fn patterns_extracted_without_options() {
        let patterns = patterns_from_rules(RULES);
        assert_eq!(
            patterns,
            [
                "ads.example.com".to_owned(),
                "spclient.wg.spotify.com/ads".to_owned()
            ]
        );
    }

    #[test]
    fn embedded_list_loads() {
        let blocker = Blocker::from_embedded().unwrap();
        assert!(blocker.is_blocked(
            "https://www.google-analytics.com/collect",
            "https://open.spotify.com/",
            "script"
        ));
        assert!(!blocker.is_blocked(
            "https://open.spotify.com/track/abc",
            "https://open.spotify.com/",
            "document"
        ));
    }
}
