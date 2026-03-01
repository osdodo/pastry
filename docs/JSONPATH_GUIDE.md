# JSONPath Search Guide (JSONPath)

Pastry provides a powerful JSONPath search feature. In the JSON viewer, you can use standard JSONPath syntax in the input box at the bottom to filter and query data.

## Basic Syntax

| Symbol | Description |
| :--- | :--- |
| `$` | Root node, represents the entire JSON document. |
| `.` | Direct child of the current node. |
| `..` | Recursive descent, searches all descendants. |
| `*` | Wildcard, matches all members or array elements. |
| `[]` | Subscript operator for array indices or object properties (with quotes). |

## Common Query Examples

The following JSON snippet is used in the examples below:

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

### 1. Get all books
`$.store.book[*]`

### 2. Get all authors
`$.store.book[*].author`

### 3. Get all prices (recursive search)
`$..price`

### 4. Get a book by index
`$.store.book[0]`

### 5. Get the first two books
`$.store.book[0:2]`

## Filters (Filter Expressions)

JSONPath supports conditional queries using `?()`.

### 1. Books with price greater than 10
`$.store.book[?(@.price > 10)]`

### 2. Books that contain an `isbn` field
`$.store.book[?(@.isbn)]`

---

## How to Run a Search
1. Enter any of the paths above in the input box at the bottom.
2. Press **Enter (Return)**.
3. The view will show only the results that match the path.
4. **Clear** the input box and press **Enter** to show the full JSON again.
