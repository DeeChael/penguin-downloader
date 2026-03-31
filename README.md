# Penguin Downloader 文档

一个模块化的音乐下载器核心。

## 文档目录

### 使用指南

- [**使用指南**](docs/使用指南.md) - 快速上手和基本使用说明

### 开发指南

- [**插件开发**](docs/插件开发.md) - 如何创建插件项目
- [**提供者开发**](docs/提供者开发.md) - 创建音乐源提供者
- [**标签器开发**](docs/标签器开发.md) - 创建元数据标签器

## 快速链接

### 如果你是库使用者

查看 [使用指南](docs/使用指南.md) 了解如何：
- 添加依赖
- 基本使用示例
- 配置选项
- 错误处理

### 如果你是插件开发者

查看以下文档了解如何开发插件：

1. [插件开发](docs/插件开发.md) - 插件项目结构和编译
2. [提供者开发](docs/提供者开发.md) - 实现 MusicProvider trait
3. [标签器开发](docs/标签器开发.md) - 实现 Tagger trait

## 项目结构

```
.
├── src/                    # 核心库源码
│   ├── core.rs            # PenguinCore
│   ├── download/          # 下载功能
│   ├── error.rs           # 错误类型
│   ├── lib.rs             # 库导出
│   ├── model/             # 数据模型
│   ├── plugin/            # 插件系统
│   ├── provider/          # 提供者 trait
│   └── tagger/            # 标签器 trait
├── Cargo.toml             # 包配置
└── docs/                  # 文档
    ├── 使用指南.md
    ├── 插件开发.md
    ├── 提供者开发.md
    └── 标签器开发.md
```

## 许可证

MIT License