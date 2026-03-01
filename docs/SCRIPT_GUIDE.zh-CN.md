# Script 使用说明

本文档提供 Pastry Script 功能的详细说明。

## 基础概念

- `input`：当前剪贴板内容。
- `output`：脚本处理后的输出结果。
- `console.log(...)`：输出调试信息。

## 内置函数

- `md5(value)`
- `sha256(value)`
- `base64_encode(value)`
- `base64_decode(value)`
- `uuid()`

## 示例

```javascript
// 处理剪贴板内容
output = base64_encode(input);

// 调试输出
console.log(uuid());
console.log(md5(input));
```

## 常见使用流程

1. 打开脚本管理器。
2. 新建或编辑脚本。
3. 通过 `input` 读取内容。
4. 将结果写入 `output`。
5. 在剪贴板条目上运行脚本。
