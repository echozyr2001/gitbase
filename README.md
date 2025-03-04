[Zh-cmn](./README-zh-cmn.md)

## 1. Core Concept Design
  
| Concept     | Corresponding GitHub Entity | Description |
|------------|----------------------------|-------------|
| Database   | GitHub Repository          | A GitBase instance corresponds to a GitHub repository |
| Collection | Directory in the repository | Similar to a database table, storing related documents |
| Document   | File in a directory        | Document content (JSON/Markdown) |
| Index      | Metadata under `_indexes/` | Index files for accelerating queries |
| Transaction | Git Commit                | Each operation corresponds to an atomic commit |

## 2. Repository Structure Design

```
my-gitbase-repo/
├── .gitbase/           # System metadata
│   ├── schemas/        # Collection structure definitions (JSON Schema)
│   └── config.json     # Global configuration
├── collections/        # Data collections
│   ├── notes/          # Example collection: Study Notes
│   │   ├── doc1.json   # Document file
│   │   ├── doc2.md
│   │   └── _indexes/   # Index directory
│   │       ├── by_tag.json
│   │       └── by_date.json
│   └── tasks/          # Another collection: Task Management
└── attachments/        # Large files (e.g., images)
```

Key Design Principles:

- **Human-readable**: The file structure is clear when browsing through the GitHub UI.
- **Machine-friendly**: Efficient queries are enabled through index files (`_indexes/*.json`).
- **Scalability**: Each collection can define its own data structure using JSON Schema.

## 3. Core API Design

| API | Corresponding GitHub Operation | Description |
|-----|--------------------------------|-------------|
| `create_database(repo_name)` | Create GitHub Repository | Initialize a GitBase instance |
| `create_collection(repo, name)` | Create Directory | Create a collection under `collections/` |
| `insert_document(repo, collection, doc_id, content)` | Create/Update File | Write a JSON/Markdown document to a collection |
| `get_document(repo, collection, doc_id)` | Read File | Read a JSON/Markdown document |
| `delete_document(repo, collection, doc_id)` | Delete File | Remove a document from the collection |
| `query_documents(repo, collection, filter)` | Read Index File | Query documents through `_indexes/` |
| `commit_transaction(repo, message)` | Git Commit | Record change history |

## 4. Bech32 Naming Rules

| Type        | Prefix (HRP) | Data Source                      | Example                    |
|------------|------------|--------------------------------|----------------------------|
| Document ID | `gb-doc`   | SHA256(document content + timestamp) | `gbdoc1qwe9acxhsh2du2d7j2r30n` |
| Collection ID | `gb-col`   | Blake3(collection name)            | `gbcol1pzx8r2dmxu0fkt63`    |
| Index ID    | `gb-idx`   | SHA256(index name + collection name) | `gbidx1ar8mfw2n6thpjz52`   |

Q: Why does the document ID use SHA256?

A: SHA256 is suitable for storing content hashes, ensuring uniqueness.

<details>
<summary>Explanation</summary>
Since document content is often large (JSON/Markdown), we want to use a cryptographically secure hash function to avoid hash collisions (i.e., different content generating the same ID). SHA256 has been extensively tested and has an extremely low collision probability, making it ideal for uniquely identifying documents.
</details>

Q: Why does the collection ID use Blake3?

A: Blake3 is faster and more efficient for generating collection IDs, reducing unnecessary computational overhead.

<details>
<summary>Explanation</summary>
Collection names are usually short (e.g., "notes", "tasks"), making them low in computational complexity. The security strength of SHA256 is not as critical for collection IDs. Blake3 is 5-10 times faster than SHA256, making it a better choice for hashing short strings.
</details>

Q: Why does the index ID use SHA256?

A: SHA256 ensures high uniqueness and stability for index IDs.

<details>
<summary>Explanation</summary>
Indexes depend on multiple documents, and SHA256 guarantees their uniqueness and stability. It prevents collisions when index names are the same but serve different purposes.
</details>
