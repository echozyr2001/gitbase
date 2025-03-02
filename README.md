## 1. 核心概念设计
  
| 概念 | GitHub 对应实体 | 说明 |
|-----|----------------|-----|
| Database | GitHub仓库 | 一个 GitBase 实例对应一个 GitHub 仓库 |
| Collection | 仓库中的目录 | 类似数据库的表，存放同类文档 |
| Document | 目录中的文件 | 文档内容（JSON/Markdown）|
| Index | _indexes/下的元数据 | 加速查询的索引文件 |
| Transaction | Git Commit | 每个操作对应一个原子提交 |

## 2. 仓库结构设计

```
my-gitbase-repo/
├── .gitbase/            # 系统元数据
│   ├── schemas/        # 集合结构定义（JSON Schema）
│   └── config.json     # 全局配置
├── collections/        # 数据集合
│   ├── notes/          # 示例集合：学习笔记
│   │   ├── doc1.json   # 文档文件
│   │   ├── doc2.md
│   │   └── _indexes/   # 索引目录
│   │       ├── by_tag.json
│   │       └── by_date.json
│   └── tasks/          # 另一个集合：任务管理
└── attachments/        # 大型文件（如图片）

```

关键设计原则：

* 人类可读：直接通过GitHub页面浏览时，文件结构清晰
* 机器友好：通过索引文件(_indexes/*.json)实现高效查询
* 扩展性：每个集合可定义独立的数据结构（通过JSON Schema）