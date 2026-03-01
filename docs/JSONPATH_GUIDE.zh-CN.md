# JSONPath 搜索指南

Pastry 提供了 JSONPath 搜索功能。在 JSON 查看器中，你可以在底部输入框里使用标准 JSONPath 语法进行筛选与查询。

## 基础语法

| 符号 | 说明 |
| :--- | :--- |
| `$` | 根节点，表示整个 JSON 文档。 |
| `.` | 当前节点的直接子节点。 |
| `..` | 递归向下，搜索所有后代节点。 |
| `*` | 通配符，匹配所有成员或数组元素。 |
| `[]` | 下标运算符，用于数组索引或对象属性（可带引号）。 |

## 常见查询示例

以下示例使用这段 JSON：

```json
{
  "store": {
    "book": [
      {
        "category": "reference",
        "author": "Nigel Rees",
        "title": "Sayings of the Century",
        "price": 8.95
      },
      {
        "category": "fiction",
        "author": "Evelyn Waugh",
        "title": "Sword of Honour",
        "price": 12.99,
        "isbn": "0-553-21311-3"
      }
    ],
    "bicycle": {
      "color": "red",
      "price": 19.95
    }
  }
}
```

### 1) 获取所有图书
`$.store.book[*]`

### 2) 获取所有作者
`$.store.book[*].author`

### 3) 获取所有价格（递归搜索）
`$..price`

### 4) 按索引获取图书
`$.store.book[0]`

### 5) 获取前两本图书
`$.store.book[0:2]`

## 过滤条件（Filter Expressions）

JSONPath 支持通过 `?()` 进行条件查询。

### 1) 查找价格大于 10 的图书
`$.store.book[?(@.price > 10)]`

### 2) 查找包含 `isbn` 字段的图书
`$.store.book[?(@.isbn)]`

---

## 如何执行搜索
1. 在底部输入框中输入上面的任一路径。
2. 按 **Enter（回车）**。
3. 视图会仅显示匹配路径的结果。
4. 清空输入框并再次按 **Enter**，即可恢复显示完整 JSON。
