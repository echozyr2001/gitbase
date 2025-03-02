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

## 3.核心 API 设计

| API | GitHub 对应操作 | 说明 |
|--|--|--|
| create_database(repo_name) | 创建 GitHub 仓库 | 初始化一个 GitBase 实例 |
| create_collection(repo, name) | 创建目录 | 在 collections/ 下创建集合 |
| insert_document(repo, collection, doc_id, content) | 创建/更新文件 | 向集合中写入 JSON/Markdown 文档 |
| get_document(repo, collection, doc_id) | 读取文件 | 读取 JSON/Markdown 文档 |
| delete_document(repo, collection, doc_id) | 删除文件 | 从集合中删除文档 |
| query_documents(repo, collection, filter) | 读取索引文件 | 通过 _indexes/ 查询文档 |
| commit_transaction(repo, message) | Git 提交 | 记录变更历史 |

## 4.Bech32 命名规则

| 类型 | 前缀（HRP）| 数据源 | 示例 |
|--|--|--|--|
| 文档 ID | gb-doc | SHA256(文档内容+时间戳) | gbdoc1qwe9acxhsh2du2d7j2r30n |
| 集合 ID | gb-col | Blake3(集合名称) | gbcol1pzx8r2dmxu0fkt63 |
| 索引 ID | gb-idx | SHA256(索引名+集合名) | gbidx1ar8mfw2n6thpjz52 |

为什么文档 ID 使用 SHA256?
	•	文档内容变化大，唯一性要求高
	•	文档 ID 由 SHA256(文档内容 + 时间戳) 生成。
	•	由于文档内容通常较大（JSON/Markdown），我们希望使用密码学安全的哈希，避免哈希碰撞（即不同内容得到相同 ID）。
	•	SHA256 经过多年验证，碰撞几率极低，适合唯一标识文档。

✅ SHA256 适用于存储内容哈希，确保唯一性。

为什么集合 ID 使用 Blake3?
	•	集合（Collection）名称短小，计算快更重要
	•	集合 ID 由 Blake3(集合名称) 计算得到。
	•	集合名称通常很短（如 "notes"、"tasks"），计算复杂度低，SHA256 的安全性对集合 ID 没有那么重要。
	•	Blake3 比 SHA256 快 5-10 倍，适合短字符串哈希。

✅ Blake3 计算快，适合快速生成集合 ID，避免不必要的计算开销。

为什么索引 ID 使用 SHA256?
	•	索引依赖于集合，与文档相关
	•	索引 ID 由 SHA256(索引名称 + 集合名称) 计算。
	•	索引依赖于多个文档，而 SHA256 保证了索引的唯一性和稳定性。
	•	避免索引名相同但作用不同时产生碰撞。

✅ SHA256 确保索引 ID 具有高唯一性和稳定性。