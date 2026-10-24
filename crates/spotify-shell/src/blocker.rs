use adblock_engine::{Blocker, FILTER_RULES, patterns_from_rules};

use crate::config::SOURCE_URL;

const INIT_SCRIPT_TEMPLATE: &str = r#"(function () {
  var patterns = __SPOTIFY_BLOCK_PATTERNS__;
  function matches(url) {
    var current = String(url).toLowerCase();
    for (var index = 0; index < patterns.length; index += 1) {
      if (current.indexOf(patterns[index]) !== -1) {
        return true;
      }
    }
    return false;
  }
  function notify(url) {
    try {
      if (window.ipc && window.ipc.postMessage) {
        window.ipc.postMessage(JSON.stringify({ blocked: url }));
      }
    } catch (ignored) {}
  }
  function absolute(input) {
    try {
      if (typeof input === "string") {
        return new URL(input, window.location.href).href;
      }
      if (input && typeof input.url === "string") {
        return new URL(input.url, window.location.href).href;
      }
    } catch (ignored) {}
    return "";
  }
  var originalFetch = window.fetch;
  window.fetch = function () {
    var url = absolute(arguments[0]);
    if (url !== "" && matches(url)) {
      notify(url);
      return Promise.reject(new Error("net::ERR_BLOCKED_BY_CLIENT"));
    }
    return originalFetch.apply(this, arguments);
  };
  var originalOpen = XMLHttpRequest.prototype.open;
  var originalSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.open = function (method, url) {
    var target = absolute(url);
    this.__spotifyAdsBlocked = target !== "" && matches(target);
    this.__spotifyAdsUrl = target;
    return originalOpen.apply(this, arguments);
  };
  XMLHttpRequest.prototype.send = function () {
    if (this.__spotifyAdsBlocked) {
      notify(this.__spotifyAdsUrl);
      var xhr = this;
      window.setTimeout(function () {
        if (typeof xhr.onerror === "function") {
          xhr.onerror(new Error("net::ERR_BLOCKED_BY_CLIENT"));
        }
      }, 0);
      return;
    }
    return originalSend.apply(this, arguments);
  };
  if (navigator && typeof navigator.sendBeacon === "function") {
    var originalBeacon = navigator.sendBeacon;
    navigator.sendBeacon = function () {
      var target = absolute(arguments[0]);
      if (target !== "" && matches(target)) {
        notify(target);
        return true;
      }
      return originalBeacon.apply(this, arguments);
    };
  }
})();"#;

pub fn should_allow_navigation(blocker: &Blocker, url: &str) -> bool {
    !blocker.is_blocked(url, SOURCE_URL, "document")
}

pub fn build_init_script() -> String {
    let patterns = patterns_from_rules(FILTER_RULES)
        .iter()
        .map(|pattern| format!("{pattern:?}"))
        .collect::<Vec<String>>()
        .join(",");
    INIT_SCRIPT_TEMPLATE.replace("__SPOTIFY_BLOCK_PATTERNS__", &format!("[{patterns}]"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = "||ads.example.com^\n||spclient.wg.spotify.com/ads^\n";

    #[test]
    fn ad_navigation_blocked() {
        let blocker = Blocker::from_rules(RULES).unwrap();
        assert!(!should_allow_navigation(
            &blocker,
            "https://ads.example.com/banner.js?x=1"
        ));
    }

    #[test]
    fn legitimate_navigation_allowed() {
        let blocker = Blocker::from_rules(RULES).unwrap();
        assert!(should_allow_navigation(
            &blocker,
            "https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQ"
        ));
        assert!(should_allow_navigation(
            &blocker,
            "https://spclient.wg.spotify.com/metadata/4uLU6hMCjMI75M1A2tKUQ"
        ));
    }

    #[test]
    fn init_script_embeds_patterns() {
        let script = build_init_script();
        assert!(!script.contains("__SPOTIFY_BLOCK_PATTERNS__"));
        assert!(script.contains("spclient.wg.spotify.com/ads"));
        assert!(script.contains("window.fetch"));
    }
}
