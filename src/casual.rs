use crate::api::TokenizerI;
use crate::regex_cache::cached_regex;

fn html_unescape(text: &str) -> String {
    let mut s = text.to_string();
    let entities: &[(&str, &str)] = &[
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
        ("&#39;", "'"),
        ("&#34;", "\""),
        ("&nbsp;", "\u{00A0}"),
        ("&pound;", "\u{00A3}"),
    ];
    for (ent, chr) in entities {
        s = s.replace(ent, chr);
    }
    let numeric_re = cached_regex(r"&#(x?)([0-9a-fA-F]+);");
    let s2 = numeric_re
        .replace_all(&s, |caps: &regex::Captures| {
            let is_hex = &caps[1] == "x" || &caps[1] == "X";
            let num_str = &caps[2];
            let num = if is_hex {
                u32::from_str_radix(num_str, 16).ok()
            } else {
                num_str.parse::<u32>().ok()
            };
            match num.and_then(char::from_u32) {
                Some(c) => c.to_string(),
                None => String::new(),
            }
        })
        .to_string();
    s2
}

fn reduce_lengthening(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let mut count = 1usize;
        while chars.peek() == Some(&c) {
            chars.next();
            count += 1;
        }
        let emit = count.min(3);
        for _ in 0..emit {
            out.push(c);
        }
    }
    out
}

fn remove_handles(text: &str) -> String {
    let re = cached_regex(r"@\w{1,15}\b");
    let mut out = String::new();
    let mut last = 0usize;
    for m in re.find_iter(text) {
        let s = m.start();
        let is_boundary = if s == 0 {
            true
        } else {
            let prev = text[..s].chars().last().unwrap_or(' ');
            !prev.is_alphanumeric()
                && !matches!(prev, '_' | '!' | '@' | '#' | '$' | '%' | '&' | '*')
        };
        if is_boundary {
            out.push_str(&text[last..s]);
            out.push(' ');
            last = m.end();
        }
    }
    out.push_str(&text[last..]);
    out
}

fn collapse_hang(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() {
            out.push(c);
            continue;
        }
        let mut count = 1usize;
        while chars.peek() == Some(&c) {
            chars.next();
            count += 1;
        }
        let emit = if count >= 4 { 3 } else { count };
        for _ in 0..emit {
            out.push(c);
        }
    }
    out
}

fn build_word_pattern(match_phone: bool) -> String {
    let emoticons = r"(?:[<>]?[:;=8][\-o\*']?[\)\]\(\[dDpP/:\}\{@\|\\]|[\)\]\(\[dDpP/:\}\{@\|\\][\-o\*']?[:;=8][<>]?|</?3)";
    let urls = r"(?:https?://[^\s<>\[\]{}()]+|[a-z0-9]+(?:[.\-][a-z0-9]+)*\.[a-z]{2,13}\b/?)";
    let phone = r"(?:\+?[01][ *\-.)]*\(?\d{3}[ *\-.)]*\d{3}[ *\-.)]*\d{4})";
    let html_tags = r"<[^>\s]+>";
    let arrows = r"[\-]+>|<[\-]+";
    let handles = r"@[\w_]+";
    let hashtags = r"\#+[\w_]+[\w'_\-]*[\w_]+";
    let emails = r"[\w.+\-]+@[\w\-]+\.(?:[\w\-]\.?)+[\w\-]";
    let flags = r"(?:[\u{1F1E6}-\u{1F1FF}]{2})";
    let words = r"(?:[^\W\d_](?:[^\W\d_]|['\-_])+[^\W\d_]|[+\-]?\d+[,/.:-]\d+[+\-]?|[\w_]+|\.(?:\s*\.){1,}|\S)";

    let mut parts: Vec<String> = Vec::new();
    parts.push(urls.to_string());
    if match_phone {
        parts.push(phone.to_string());
    }
    parts.push(emoticons.to_string());
    parts.push(html_tags.to_string());
    parts.push(arrows.to_string());
    parts.push(handles.to_string());
    parts.push(hashtags.to_string());
    parts.push(emails.to_string());
    parts.push(flags.to_string());
    parts.push(words.to_string());
    format!("({})", parts.join("|"))
}

pub struct TweetTokenizer {
    pub preserve_case: bool,
    pub reduce_len: bool,
    pub strip_handles: bool,
    pub match_phone_numbers: bool,
}

impl Default for TweetTokenizer {
    fn default() -> Self {
        Self {
            preserve_case: true,
            reduce_len: false,
            strip_handles: false,
            match_phone_numbers: true,
        }
    }
}

impl TweetTokenizer {
    pub fn new(
        preserve_case: bool,
        reduce_len: bool,
        strip_handles: bool,
        match_phone_numbers: bool,
    ) -> Self {
        Self {
            preserve_case,
            reduce_len,
            strip_handles,
            match_phone_numbers,
        }
    }

    fn word_re(&self) -> regex::Regex {
        let pat = build_word_pattern(self.match_phone_numbers);
        cached_regex(&pat)
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut s = html_unescape(text);
        if self.strip_handles {
            s = remove_handles(&s);
        }
        if self.reduce_len {
            s = reduce_lengthening(&s);
        }
        let safe = collapse_hang(&s);

        let re = self.word_re();
        let mut words: Vec<String> = re
            .find_iter(&safe)
            .map(|m| m.as_str().to_string())
            .collect();

        if !self.preserve_case {
            let emoticon_re =
                cached_regex(r"(?i)(?:[<>]?[:;=8][\-o\*']?[\)\]\(\[dDpP/:\}\{@\|\\]|[\)\]\(\[dDpP/:\}\{@\|\\][\-o\*']?[:;=8][<>]?|</?3)");
            words = words
                .into_iter()
                .map(|w| {
                    if emoticon_re.is_match(&w) {
                        w
                    } else {
                        w.to_lowercase()
                    }
                })
                .collect();
        }
        words
    }
}

impl TokenizerI for TweetTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        TweetTokenizer::tokenize(self, s)
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        let toks = self.tokenize(s);
        crate::util::align_tokens(&toks, s)
    }
}

pub fn casual_tokenize(
    text: &str,
    preserve_case: bool,
    reduce_len: bool,
    strip_handles: bool,
    match_phone_numbers: bool,
) -> Vec<String> {
    TweetTokenizer::new(
        preserve_case,
        reduce_len,
        strip_handles,
        match_phone_numbers,
    )
    .tokenize(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let t = TweetTokenizer::default();
        let toks = t.tokenize("Hello world :)");
        assert!(toks.contains(&"Hello".to_string()));
        assert!(toks.contains(&":)".to_string()));
    }
    #[test]
    fn url() {
        let t = TweetTokenizer::default();
        let toks = t.tokenize("Visit https://example.com today");
        assert!(toks.iter().any(|x| x.contains("example.com")));
    }
    #[test]
    fn hashtag() {
        let t = TweetTokenizer::default();
        let toks = t.tokenize("I love #rust!");
        assert!(toks.iter().any(|x| x.starts_with('#')));
    }
    #[test]
    fn handle() {
        let t = TweetTokenizer::default();
        let toks = t.tokenize("Hey @user how are you?");
        assert!(toks.contains(&"@user".to_string()));
    }
    #[test]
    fn reduce_len() {
        let t = TweetTokenizer::new(true, true, false, true);
        let toks = t.tokenize("coooool");
        assert!(toks.contains(&"coool".to_string()));
    }
    #[test]
    fn strip_handles() {
        let t = TweetTokenizer::new(true, false, true, true);
        let toks = t.tokenize("@remy Hello");
        assert!(!toks.contains(&"@remy".to_string()));
    }
}
