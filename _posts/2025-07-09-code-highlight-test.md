---
layout: post
title: "代码高亮测试"
date: 2025-07-09
tags: [测试, 代码高亮]
---

# 代码高亮效果展示

这篇文章用来测试不同编程语言的代码高亮效果。

## Rust 代码

```rust
fn main() {
    let greeting = "Hello, world!";
    println!("{}", greeting);
    
    // 示例结构体
    struct Person {
        name: String,
        age: u32,
    }
    
    let person = Person {
        name: String::from("Alice"),
        age: 30,
    };
    
    println!("Name: {}, Age: {}", person.name, person.age);
}
```

## JavaScript 代码

```javascript
// 异步函数示例
async function fetchData(url) {
    try {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Error fetching data:', error);
        throw error;
    }
}

// 使用示例
fetchData('https://api.example.com/data')
    .then(data => console.log(data))
    .catch(error => console.error(error));
```

## Python 代码

```python
import asyncio
from typing import List, Optional

class DataProcessor:
    def __init__(self, name: str):
        self.name = name
        self.data: List[int] = []
    
    async def process_data(self, numbers: List[int]) -> Optional[float]:
        """处理数据并返回平均值"""
        if not numbers:
            return None
        
        # 模拟异步处理
        await asyncio.sleep(0.1)
        
        self.data.extend(numbers)
        return sum(numbers) / len(numbers)

# 使用示例
async def main():
    processor = DataProcessor("测试处理器")
    result = await processor.process_data([1, 2, 3, 4, 5])
    print(f"平均值: {result}")

if __name__ == "__main__":
    asyncio.run(main())
```

## HTML/CSS 代码

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>示例页面</title>
    <style>
        .container {
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        
        .highlight {
            background-color: #f6f8fa;
            border-left: 4px solid #0366d6;
            padding: 16px;
            margin: 16px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>欢迎来到我的博客</h1>
        <div class="highlight">
            <p>这是一个高亮的内容块。</p>
        </div>
    </div>
</body>
</html>
```

## YAML 配置

```yaml
# Jekyll 配置示例
title: "技术博客"
description: "分享编程知识"
baseurl: ""
url: "https://example.github.io"

# 构建设置
markdown: kramdown
highlighter: rouge

# 插件
plugins:
  - jekyll-feed
  - jekyll-sitemap

# 默认设置
defaults:
  - scope:
      path: ""
      type: "posts"
    values:
      layout: "post"
      author: "博主"
```

## JSON 数据

```json
{
  "name": "示例数据",
  "version": "1.0.0",
  "description": "这是一个JSON示例",
  "keywords": ["json", "示例", "测试"],
  "author": {
    "name": "作者姓名",
    "email": "author@example.com"
  },
  "dependencies": {
    "express": "^4.18.0",
    "mongoose": "^6.0.0"
  },
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest"
  }
}
```

## Bash 脚本

```bash
#!/bin/bash

# 简单的部署脚本
set -e

echo "开始部署..."

# 检查Git状态
if [[ -n $(git status --porcelain) ]]; then
    echo "错误：工作目录不干净，请先提交更改"
    exit 1
fi

# 构建项目
echo "构建项目..."
npm run build

# 部署到服务器
echo "部署到服务器..."
rsync -avz --delete build/ user@server:/var/www/html/

echo "部署完成！"
```

## 行内代码

在文本中，我们经常需要引用一些代码片段，比如 `Arc::clone(&arc)` 函数，或者 `let variable = "value"` 这样的变量声明。

## 总结

以上展示了各种编程语言的代码高亮效果。现在的样式应该：

1. **简洁明了** - 使用GitHub风格的配色
2. **易于阅读** - 合适的字体和行高
3. **功能齐全** - 支持复制按钮和滚动条
4. **响应式** - 在不同设备上都能良好显示
