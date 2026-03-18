//! 字段处理转化
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use anyhow::Context;
use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::core::{ParsedRecord, Parser};

/// nginx 日志解析
pub struct NginxParser {
    regex : Regex,
}

/// 预编译正则（性能优化）
static NGINX_LOG_REGEX: Lazy<Regex> = Lazy::new(|| {
    // 分组: ip, ident, auth_user, datetime, request, status, bytes, referer, user_agent
    Regex::new(
        r#"^(?P<ip>[\d.]+)\s+-\s+(?P<auth>\S+)\s+\[(?P<datetime>[^\]]+)\]\s+"(?P<request>[^"]*)"\s+(?P<status>\d{3})\s+(?P<bytes>\d+|-)\s+"(?P<referer>[^"]*)"\s+"(?P<user_agent>[^"]*)""#
    ).expect("正则编译失败")
});


impl NginxParser {
    pub fn new() -> Self {
        NginxParser {
            regex: NGINX_LOG_REGEX.clone(),
        }
    }

    /// 从 request 字段提取 method, url, protocol
    fn parse_request(request: &str) -> (String, String, String) {
        let parts: Vec<&str> = request.split_whitespace().collect();
        match parts.len() {
            3 => (parts[0].to_string(), parts[1].to_string(), parts[2].to_string()),
            2 => (parts[0].to_string(), parts[1].to_string(), "".to_string()),
            1 => (parts[0].to_string(),  "".to_string(), "".to_string()),
            _ => ("".to_string(), "".to_string(), "".to_string())
        }
    }

    /// 解析nginx时间格式
    fn parse_time(datetime: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        DateTime::parse_from_str(datetime, "%d/%b/%Y:%H:%M:%S %z")
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }
}

impl Parser for NginxParser {
    fn parse(&self, line: &str) -> anyhow::Result<ParsedRecord> {
        let caps = self
            .regex
            .captures(line).context(format!("日志解析失败行：{}",  line))?;

        let ip = caps.name("ip").map(|m| m.as_str()).unwrap_or_default();
        let auth = caps.name("auth").map(|m| m.as_str()).unwrap_or_default();
        let datetime = caps.name("datetime").map(|m| m.as_str()).unwrap_or_default();
        let request = caps.name("request").map(|m| m.as_str()).unwrap_or_default();
        let status = caps.name("status").map(|m| m.as_str()).unwrap_or_default();
        let bytes = caps.name("bytes").map(|m| m.as_str()).unwrap_or_default();
        let referer = caps.name("referer").map(|m| m.as_str()).unwrap_or_default();
        let user_agent = caps.name("user_agent").map(|m| m.as_str()).unwrap_or_default();

        let (method, url, protocol) = Self::parse_request(request);
        let bytes_num  = if bytes == "-" {
            "0"
        } else {
            bytes
        };

        /// 构建字段列表
        let fields = vec![
            ("ip".to_string(), ip.to_string()),
            ("auth_user".to_string(), auth.to_string()),
            ("method".to_string(), method),
            ("url".to_string(), url),
            ("protocol".to_string(), protocol),
            ("status".to_string(), status.to_string()),
            ("bytes".to_string(), bytes_num.to_string()),
            ("referer".to_string(), referer.to_string()),
            ("user_agent".to_string(), user_agent.to_string()),
            ("raw".to_string(), line.to_string()), // 保留原始行
        ];

        Ok(ParsedRecord{timestamp: Self::parse_time(datetime), fields})
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_parse_nginx_log() {
        let parser = NginxParser::new();
        let line = r#"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326 "http://www.example.com/start.html" "Mozilla/4.08 [en] (Win98; I ;Nav)""#;

        let record = parser.parse(line).unwrap();
        assert_eq!(record.timestamp.is_some(), true);

        let get_field = |name :&str| -> String {
            record.fields.iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone()).unwrap_or_default()
        };

        assert_eq!(get_field("ip"), "127.0.0.1");
        assert_eq!(get_field("method"), "GET");
        assert_eq!(get_field("url"), "/apache_pb.gif");
        assert_eq!(get_field("status"), "200");
        assert_eq!(get_field("bytes"), "2326");

    }
    #[test]
    fn test_parse_invalid_log() {
        let parser = NginxParser::new();
        let line = "this is not a valid nginx log";

        assert!(parser.parse(line).is_err());
    }

}