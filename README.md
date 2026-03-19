# rust-log-etl

高性能日志 ETL（Extract-Transform-Load）工具，使用 Rust 编写。支持多种日志格式解析、灵活的数据转换和多种输出方式。

## ✨ 特性

- 🚀 **高性能**：基于 Tokio 异步运行时，充分利用多核 CPU
- 🔧 **灵活的解析器**：支持 Nginx、JSON、自定义正则等多种日志格式
- 🎯 **强大的转换器**：支持过滤、富化、字段提取等多种转换操作
- 📦 **多种输出格式**：支持 JSON、CSV、原始文本等格式
- ⚙️ **配置驱动**：使用 TOML 配置文件，简单易用
- 🛠️ **双模式运行**：支持配置文件模式和快速命令行模式

## 📦 安装

### 从源码编译

```bash
git clone https://github.com/yourusername/rust-log-etl.git
cd rust-log-etl
cargo build --release
```

编译后的二进制文件位于 `target/release/rust-log-etl`

## 🚀 快速开始

### 模式一：快速命令行模式

适合简单场景，无需配置文件：

```bash
# 处理 Nginx 日志，输出为 JSON 格式
rust-log-etl run \
    --input /var/log/nginx/access.log \
    --output /tmp/output.json \
    --parser nginx \
    --format json

# 使用默认参数（workers=4）
rust-log-etl run -i access.log -o output.json
```

### 模式二：配置文件模式

适合复杂场景，支持过滤器、富化器等高级功能：

```bash
rust-log-etl config --config log_config.toml
```

## 📖 配置文件说明

### 完整配置示例

```toml
[global]
# 工作线程数
workers = 4
# 批量处理大小
batch_size = 1000

[input]
type = "file"
path = "/var/log/nginx/access.log"

[parser]
type = "nginx"
# 或者使用自定义正则
# type = "regex"
# pattern = '^(?P<ip>[\d.]+)...'

[[transformers]]
type = "filter"
field = "status"
operator = "not_in"
value = ["404", "500"]

[[transformers]]
type = "enrich"
field = "hour"
source_field = "timestamp"
enrich_type = "time_format"
format = "%H"

[sink]
type = "file"
path = "/var/log/output/processed.log"
format = "json"
# 可选：日志轮转配置
# rotate_size = "100MB"
# rotate_time = "1h"
```

### 配置项详解

#### 1. 全局配置 `[global]`

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `workers` | usize | 4 | 工作线程数 |
| `batch_size` | usize | 1000 | 批量处理数量 |

#### 2. 输入配置 `[input]`

```toml
[input]
type = "file"
path = "/path/to/log/file"
```

支持的输入类型：
- `file`: 从文件读取

#### 3. 解析器配置 `[parser]`

**Nginx 解析器**：
```toml
[parser]
type = "nginx"
# 可选：自定义正则表达式
# custom_regex = "^.*$"
```

**JSON 解析器**：
```toml
[parser]
type = "json"
# 可选：字段映射
field_map = { "ts" = "timestamp", "lvl" = "level" }
```

**正则解析器**：
```toml
[parser]
type = "regex"
pattern = '^(?P<field1>...) (?P<field2>...)'
fields = ["field1", "field2"]
```

#### 4. 转换器配置 `[[transformers]]`

**过滤器（Filter）**：

```toml
# 等于
[[transformers]]
type = "filter"
field = "status"
operator = "eq"
value = "200"

# 大于
[[transformers]]
type = "filter"
field = "bytes"
operator = "gt"
value = 1000

# 在列表中
[[transformers]]
type = "filter"
field = "status"
operator = "in"
value = ["200", "201", "302"]

# 不在列表中
[[transformers]]
type = "filter"
field = "status"
operator = "not_in"
value = ["404", "500"]

# 正则匹配
[[transformers]]
type = "filter"
field = "url"
operator = "regex"
value = "^/api/.*$"

# 包含
[[transformers]]
type = "filter"
field = "message"
operator = "contains"
value = "error"
```

支持的运算符：
- `eq`: 等于
- `ne`: 不等于
- `gt`: 大于
- `lt`: 小于
- `in`: 在列表中
- `not_in`: 不在列表中
- `contains`: 包含
- `regex`: 正则匹配

**富化器（Enrich）**：

```toml
# 时间格式化
[[transformers]]
type = "enrich"
field = "hour"
source_field = "timestamp"
enrich_type = "time_format"
format = "%H"

# URL 解析
[[transformers]]
type = "enrich"
field = "domain"
source_field = "url"
enrich_type = "url_parse"
part = "domain"

# IP 地理位置（阶段 3）
[[transformers]]
type = "enrich"
field = "country"
source_field = "ip"
enrich_type = "ip_geo"
```

#### 5. 输出配置 `[sink]`

**文件输出**：
```toml
[sink]
type = "file"
path = "/path/to/output.log"
format = "json"  # json, csv, raw, delimited
```

支持的输出格式：
- `json`: JSON 格式
- `csv`: CSV 格式
- `raw`: 原始字符串
- `delimited`: 自定义分隔符（需额外配置 `delimiter` 和 `fields`）

**Kafka 输出**（开发中）：
```toml
[sink]
type = "kafka"
brokers = ["localhost:9092"]
topic = "logs"
batch_size = 1000
```

## 💡 使用示例

### 示例 1：过滤错误日志

只保留非 404 和 500 状态的请求：

```toml
[parser]
type = "nginx"

[[transformers]]
type = "filter"
field = "status"
operator = "not_in"
value = ["404", "500"]

[sink]
type = "file"
path = "filtered.json"
format = "json"
```

### 示例 2：提取小时字段

从时间戳中提取小时：

```toml
[parser]
type = "nginx"

[[transformers]]
type = "enrich"
field = "hour"
source_field = "timestamp"
enrich_type = "time_format"
format = "%H"

[sink]
type = "file"
path = "by_hour.json"
format = "json"
```

### 示例 3：JSON 日志字段重映射

```toml
[parser]
type = "json"
field_map = { "ts" = "timestamp", "lvl" = "level", "msg" = "message" }

[sink]
type = "file"
path = "normalized.json"
format = "json"
```

## 🧪 测试

```bash
# 运行测试
cargo test

# 使用测试数据
rust-log-etl run \
    --input test_data/nginx_access.log \
    --output /tmp/test_output.json \
    --parser nginx
```

## 🏗️ 架构设计

```
rust-log-etl
├── Parser      # 解析层：将原始日志转换为结构化数据
│   ├── NginxParser
│   ├── JsonParser
│   └── RegexParser
├── Transformer # 转换层：过滤、富化、字段处理
│   ├── FilterTransformer
│   └── EnrichTransformer
└── Sink        # 输出层：写入目标存储
    ├── FileSink
    └── KafkaSink (TODO)
```

## 📝 开发计划

- [ ] Kafka Sink 支持
- [ ] IP 地理位置查询
- [ ] 更多日志格式支持（Apache、Syslog 等）
- [ ] 实时流式处理
- [ ] Prometheus 指标导出
- [ ] Web UI 监控界面

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 👥 作者

yuanhu <your.email@example.com>
