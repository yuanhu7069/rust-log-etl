//! 处理 filter
//!
//! @author yuanhu
//! @created 2026/3/13 13:56
//! @version 1.0.0

use crate::config::{FilterOperator, FilterValue};
use anyhow::Result;
use regex::Regex;
use toml::map;
use crate::core::{ParsedRecord, Transformer};

pub struct FilterTransformer {
    field: String,
    operator: FilterOperator,
    value: FilterValue,

    // 缓存正则操作
    regex: Option<Regex>,
}

impl FilterTransformer {
    pub fn new(field: String, operator: FilterOperator, value: FilterValue) -> Result<Self> {
        let regex = match (&operator, &value) {
            (FilterOperator::Regex, FilterValue::Single(pattern)) => {
                Some(Regex::new(pattern).map_err(|e| anyhow::anyhow!("无效规则：{}", 2))?)
            }
            _ => None,
        };
        Ok(Self {field, operator, value, regex})
    }

    /// 记录中提取字段值
    fn get_field_value<'a>(&self, record: &'a ParsedRecord) -> Option<&'a str> {
        record.fields.iter()
            .find(|(k, _)| k == &self.field)
            .map(|(_, v)| v.as_str())
    }

    /// 执行过滤判断
    fn should_keep(&self, record: &ParsedRecord) -> bool {
        let field_value = match self.get_field_value(&record) {
            Some(v) => v,
            None => return true,
        };

        match &self.operator {
            FilterOperator::Eq => self.compare_equal(field_value),
            FilterOperator::Ne => !self.compare_equal(field_value),
            FilterOperator::Gt => self.compare_greater(field_value),
            FilterOperator::Lt => self.compare_less(field_value),
            FilterOperator::In => self.compare_in(field_value),
            FilterOperator::NotIn => !self.compare_in(field_value),
            FilterOperator::Contains => self.compare_contain(field_value),
            FilterOperator::Regex => self.compare_regex(field_value),
            _ => false,
        }
    }

    /// 等于比较逻辑
    fn compare_equal(&self, field_value: &str) -> bool {
        match &self.value {
            FilterValue::Single(expected) => field_value == expected.as_str(),
            FilterValue::Number(n) => field_value.parse::<f64>().ok() == Some(*n),
            FilterValue::List(_) => false,
        }
    }

    /// 大于比较逻辑
    fn compare_greater(&self, field_value: &str) -> bool {
        let field_num = match field_value.parse::<f64>() {
            Ok(n) => n,
            Err(_) => return false,
        };
        match &self.value {
            FilterValue::Number(n) => field_num > *n,
            FilterValue::Single(s) => s.parse::<f64>().map(|n| field_num > n).unwrap_or(false),
            _ => false,
        }
    }

    /// 小于比较实现
    fn compare_less(&self, field_value: &str) -> bool {
        let field_num = match field_value.parse::<f64>() {
            Ok(n) => n,
            Err(_) => return false,
        };

        match &self.value {
            FilterValue::Number(n) => field_num < *n,
            FilterValue::Single(s) => s.parse::<f64>().map(|n| field_num < n).unwrap_or(false),
            _ => false,
        }
    }

    /// 在列表中逻辑
    fn compare_in(&self, field_value: &str) -> bool {
        match &self.value {
            FilterValue::List(list) => list.iter().any(|item| item == field_value),
            FilterValue::Single(s) => s == field_value,
            _ => false,
        }
    }

    /// 包含逻辑实现
    fn compare_contain(&self, field_value: &str) -> bool {
        match &self.value {
            FilterValue::Single(substr) => field_value.contains(substr),
            FilterValue::List(list) => list.iter().any(|item| item.contains(field_value)),
            _ => false,
        }
    }

    fn compare_regex(&self, field_value: &str) -> bool {
        match &self.regex {
            Some(re) => re.is_match(field_value),
            None => false,
        }
    }

}

impl Transformer for FilterTransformer {
    fn transform(&self, record: ParsedRecord) -> Result<Option<ParsedRecord>> {
        if self.should_keep(&record) {
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
}



