use serde_json::Value;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Line {
    pub text: String,
    pub path: Option<String>,
    pub collapsed: bool,
}

pub fn render_json_lines(
    content: &str,
    parsed: &Option<Value>,
    collapsed: &HashSet<String>,
) -> Vec<Line> {
    if let Some(parsed) = parsed {
        let mut out = Vec::new();
        render_value(parsed, "", 0, true, collapsed, &mut out);
        out
    } else {
        vec![Line {
            text: content.to_string(),
            path: None,
            collapsed: false,
        }]
    }
}

fn render_value(
    value: &Value,
    path: &str,
    indent: usize,
    is_root: bool,
    collapsed: &HashSet<String>,
    out: &mut Vec<Line>,
) {
    match value {
        Value::Object(map) => {
            if is_root {
                let open = format!("{}{{", "  ".repeat(indent));
                out.push(Line {
                    text: open,
                    path: Some(path.to_string()),
                    collapsed: collapsed.contains(path),
                });
                if collapsed.contains(path) {
                    let sum = format!("{}… {} keys {}", "  ".repeat(indent + 1), map.len(), " ");
                    out.push(Line {
                        text: sum,
                        path: None,
                        collapsed: false,
                    });
                    let close = format!("{}}}", "  ".repeat(indent));
                    out.push(Line {
                        text: close,
                        path: None,
                        collapsed: false,
                    });
                    return;
                }
                for (i, (k, v)) in map.iter().enumerate() {
                    let key = format!("{}\"{}\": ", "  ".repeat(indent + 1), k);
                    match v {
                        Value::Object(_) | Value::Array(_) => {
                            let child_path = if path.is_empty() {
                                k.clone()
                            } else {
                                format!("{}.{}", path, k)
                            };
                            let open = match v {
                                Value::Object(_) => "{",
                                _ => "[",
                            };
                            let collapsed_here = collapsed.contains(&child_path);
                            out.push(Line {
                                text: format!("{}{}", key, open),
                                path: Some(child_path.clone()),
                                collapsed: collapsed_here,
                            });
                            if collapsed_here {
                                let summary = match v {
                                    Value::Object(o) => format!(
                                        "{}… {} keys {}",
                                        "  ".repeat(indent + 2),
                                        o.len(),
                                        " "
                                    ),
                                    Value::Array(a) => format!(
                                        "{}… {} items {}",
                                        "  ".repeat(indent + 2),
                                        a.len(),
                                        " "
                                    ),
                                    _ => String::new(),
                                };
                                out.push(Line {
                                    text: summary,
                                    path: None,
                                    collapsed: false,
                                });
                                let close = match v {
                                    Value::Object(_) => format!(
                                        "{}}}{}",
                                        "  ".repeat(indent + 1),
                                        if i + 1 < map.len() { "," } else { "" }
                                    ),
                                    _ => format!(
                                        "{}]{}",
                                        "  ".repeat(indent + 1),
                                        if i + 1 < map.len() { "," } else { "" }
                                    ),
                                };
                                out.push(Line {
                                    text: close,
                                    path: None,
                                    collapsed: false,
                                });
                            } else {
                                render_value(v, &child_path, indent + 2, false, collapsed, out);
                                let close = match v {
                                    Value::Object(_) => {
                                        format!("{}}}", "  ".repeat(indent + 1))
                                    }
                                    _ => format!("{}]", "  ".repeat(indent + 1)),
                                };
                                if i + 1 < map.len() {
                                    out.push(Line {
                                        text: format!("{},", close),
                                        path: None,
                                        collapsed: false,
                                    });
                                } else {
                                    out.push(Line {
                                        text: close,
                                        path: None,
                                        collapsed: false,
                                    });
                                }
                            }
                        }
                        _ => {
                            let val_text = value_to_text(v);
                            if i + 1 < map.len() {
                                out.push(Line {
                                    text: format!("{}{}{}", key, val_text, ","),
                                    path: None,
                                    collapsed: false,
                                });
                            } else {
                                out.push(Line {
                                    text: format!("{}{}", key, val_text),
                                    path: None,
                                    collapsed: false,
                                });
                            }
                        }
                    }
                }
                let close = format!("{}}}", "  ".repeat(indent));
                out.push(Line {
                    text: close,
                    path: None,
                    collapsed: false,
                });
            } else {
                let open = format!("{}{{", "  ".repeat(indent));
                out.push(Line {
                    text: open,
                    path: Some(path.to_string()),
                    collapsed: collapsed.contains(path),
                });
                if collapsed.contains(path) {
                    let sum = format!("{}… {} keys {}", "  ".repeat(indent + 1), map.len(), " ");
                    out.push(Line {
                        text: sum,
                        path: None,
                        collapsed: false,
                    });
                    let close = format!("{}}}", "  ".repeat(indent));
                    out.push(Line {
                        text: close,
                        path: None,
                        collapsed: false,
                    });
                    return;
                }
                for (i, (k, v)) in map.iter().enumerate() {
                    let key = format!("{}\"{}\": ", "  ".repeat(indent + 1), k);
                    match v {
                        Value::Object(_) | Value::Array(_) => {
                            let child_path = if path.is_empty() {
                                k.clone()
                            } else {
                                format!("{}.{}", path, k)
                            };
                            let open = match v {
                                Value::Object(_) => "{",
                                _ => "[",
                            };
                            let collapsed_here = collapsed.contains(&child_path);
                            out.push(Line {
                                text: format!("{}{}", key, open),
                                path: Some(child_path.clone()),
                                collapsed: collapsed_here,
                            });
                            if collapsed_here {
                                let summary = match v {
                                    Value::Object(o) => format!(
                                        "{}… {} keys {}",
                                        "  ".repeat(indent + 2),
                                        o.len(),
                                        " "
                                    ),
                                    Value::Array(a) => format!(
                                        "{}… {} items {}",
                                        "  ".repeat(indent + 2),
                                        a.len(),
                                        " "
                                    ),
                                    _ => String::new(),
                                };
                                out.push(Line {
                                    text: summary,
                                    path: None,
                                    collapsed: false,
                                });
                                let close = match v {
                                    Value::Object(_) => format!(
                                        "{}}}{}",
                                        "  ".repeat(indent + 1),
                                        if i + 1 < map.len() { "," } else { "" }
                                    ),
                                    _ => format!(
                                        "{}]{}",
                                        "  ".repeat(indent + 1),
                                        if i + 1 < map.len() { "," } else { "" }
                                    ),
                                };
                                out.push(Line {
                                    text: close,
                                    path: None,
                                    collapsed: false,
                                });
                            } else {
                                render_value(v, &child_path, indent + 2, false, collapsed, out);
                                let close = match v {
                                    Value::Object(_) => {
                                        format!("{}}}", "  ".repeat(indent + 1))
                                    }
                                    _ => format!("{}]", "  ".repeat(indent + 1)),
                                };
                                if i + 1 < map.len() {
                                    out.push(Line {
                                        text: format!("{},", close),
                                        path: None,
                                        collapsed: false,
                                    });
                                } else {
                                    out.push(Line {
                                        text: close,
                                        path: None,
                                        collapsed: false,
                                    });
                                }
                            }
                        }
                        _ => {
                            let val_text = value_to_text(v);
                            if i + 1 < map.len() {
                                out.push(Line {
                                    text: format!("{}{}{}", key, val_text, ","),
                                    path: None,
                                    collapsed: false,
                                });
                            } else {
                                out.push(Line {
                                    text: format!("{}{}", key, val_text),
                                    path: None,
                                    collapsed: false,
                                });
                            }
                        }
                    }
                }
                let close = format!("{}}}", "  ".repeat(indent));
                out.push(Line {
                    text: close,
                    path: None,
                    collapsed: false,
                });
            }
        }
        Value::Array(arr) => {
            let open = format!("{}[", "  ".repeat(indent));
            out.push(Line {
                text: open,
                path: Some(path.to_string()),
                collapsed: collapsed.contains(path),
            });
            if collapsed.contains(path) {
                let sum = format!("{}… {} items {}", "  ".repeat(indent + 1), arr.len(), " ");
                out.push(Line {
                    text: sum,
                    path: None,
                    collapsed: false,
                });
                let close = format!("{}]", "  ".repeat(indent));
                out.push(Line {
                    text: close,
                    path: None,
                    collapsed: false,
                });
                return;
            }
            for (i, v) in arr.iter().enumerate() {
                match v {
                    Value::Object(_) | Value::Array(_) => {
                        let child_path = format!("{}[{}]", path, i);
                        let open = match v {
                            Value::Object(_) => "{",
                            _ => "[",
                        };
                        let collapsed_here = collapsed.contains(&child_path);
                        out.push(Line {
                            text: format!("{}{}", "  ".repeat(indent + 1), open),
                            path: Some(child_path.clone()),
                            collapsed: collapsed_here,
                        });
                        if collapsed_here {
                            let summary = match v {
                                Value::Object(o) => {
                                    format!("{}… {} keys {}", "  ".repeat(indent + 2), o.len(), " ")
                                }
                                Value::Array(a) => format!(
                                    "{}… {} items {}",
                                    "  ".repeat(indent + 2),
                                    a.len(),
                                    " "
                                ),
                                _ => String::new(),
                            };
                            let close = match v {
                                Value::Object(_) => format!(
                                    "{}}}{}",
                                    "  ".repeat(indent + 1),
                                    if i + 1 < arr.len() { "," } else { "" }
                                ),
                                _ => format!(
                                    "{}]{}",
                                    "  ".repeat(indent + 1),
                                    if i + 1 < arr.len() { "," } else { "" }
                                ),
                            };
                            out.push(Line {
                                text: summary,
                                path: None,
                                collapsed: false,
                            });
                            out.push(Line {
                                text: close,
                                path: None,
                                collapsed: false,
                            });
                        } else {
                            render_value(v, &child_path, indent + 2, false, collapsed, out);
                            let close = match v {
                                Value::Object(_) => {
                                    format!("{}}}", "  ".repeat(indent + 1))
                                }
                                _ => format!("{}]", "  ".repeat(indent + 1)),
                            };
                            if i + 1 < arr.len() {
                                out.push(Line {
                                    text: format!("{},", close),
                                    path: None,
                                    collapsed: false,
                                });
                            } else {
                                out.push(Line {
                                    text: close,
                                    path: None,
                                    collapsed: false,
                                });
                            }
                        }
                    }
                    _ => {
                        let val_text = format!("{}{}", "  ".repeat(indent + 1), value_to_text(v));
                        if i + 1 < arr.len() {
                            out.push(Line {
                                text: format!("{},", val_text),
                                path: None,
                                collapsed: false,
                            });
                        } else {
                            out.push(Line {
                                text: val_text,
                                path: None,
                                collapsed: false,
                            });
                        }
                    }
                }
            }
            let close = format!("{}]", "  ".repeat(indent));
            out.push(Line {
                text: close,
                path: None,
                collapsed: false,
            });
        }
        _ => {
            let text = format!("{}{}", "  ".repeat(indent), value_to_text(value));
            out.push(Line {
                text,
                path: None,
                collapsed: false,
            });
        }
    }
}

fn value_to_text(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s),
        Value::Number(n) => format!("{}", n),
        Value::Bool(b) => format!("{}", b),
        Value::Null => "null".to_string(),
        Value::Object(_) => "{".to_string(),
        Value::Array(_) => "[".to_string(),
    }
}
