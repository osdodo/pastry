# Script Guide

This document explains how to use Pastry scripts in detail.

## Basics

- `input`: the current clipboard content.
- `output`: the processed result to return.
- `console.log(...)`: print debug output.

## Built-in Helpers

- `md5(value)`
- `sha256(value)`
- `base64_encode(value)`
- `base64_decode(value)`
- `uuid()`

## Example

```javascript
// transform clipboard content
output = base64_encode(input);

// debug info
console.log(uuid());
console.log(md5(input));
```

## Typical Workflow

1. Open Script Manager.
2. Create or edit a script.
3. Read data from `input`.
4. Write final result to `output`.
5. Run the script on a clipboard item.
