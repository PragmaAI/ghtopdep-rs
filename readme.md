# ghtopdep-rs: GitHub Dependents Analyzer

A fast, efficient Rust tool to find and analyze repositories that depend on a GitHub project.

## Features

- Find repositories that depend on a specific GitHub project
- Sort dependents by star count
- Filter by minimum stars
- Show package or repository dependents
- Multiple output formats (text, table, JSON)
- Parallel processing for fetching repository descriptions
- Efficient caching with compression
- Retry mechanism with exponential backoff

## Installation

### From Source

# Clone the repository
git clone https://github.com/yourusername/ghtopdep-rs.git
cd ghtopdep-rs

# Build with Cargo
cargo build --release

# The binary will be available at target/release/ghtopdep-rs
Usage
Command Line Options
| Option | Description | Default |
|--------|-------------|---------|
| `--rows N` | Number of top dependents to show | 10 |
| `--minstar N` | Minimum number of stars | 0 |
| `--max_pages N` | Maximum number of pages to fetch | 100 |
| `--packages` | Show package dependents instead of repositories | false |
| `--description` | Show repository descriptions | false |
| `--no-cache` | Disable caching | false |
| `--format FORMAT` | Output format (text, json, table) | table |
| `--table` | Use table output format (shorthand for --format table) | false |
Examples
Basic Usage
With Minimum Stars
bash
$ ghtopdep-rs near/near-sdk-rs --minstar 200 --rows 5
Fetching repository dependents for near/near-sdk-rs...
Found 2510 total dependents
[00:27:35] [██████████████████████████████████████████████████] 100% (2510/2510) [00:00:00]
Download complete

Sorting 2510 repositories by star count...
+------------------------------------------------+---------+
| url                                            | stars   |
+------------------------------------------------+---------+
| https://github.com/near/nearcore               | 2.4K    |
| https://github.com/wormhole-foundation/wormhole| 1.7K    |
| https://github.com/skyward-finance/contracts   | 766     |
| https://github.com/ref-finance/ref-contracts   | 345     |
| https://github.com/NearSocial/social-db        | 301     |
+------------------------------------------------+---------+
found 2510 repositories
found 8 repositories with more than 200 stars
Completed in 27.35 seconds

## Performance

ghtopdep-rs is significantly faster and more memory-efficient than similar Python tools:

| Metric | ghtopdep-rs | Python equivalent |
|--------|------------|-------------------|
| Memory usage | ~15 MB | ~47 MB |
| CPU time | ~0.2s | ~0.5s |
| With descriptions | ~0.8s | ~1.5s |

## Caching

By default, ghtopdep-rs caches GitHub responses for 24 hours to reduce API calls and improve performance. Use the `--no-cache` flag to always fetch fresh data.

## License

MIT

## Testing

The repository includes a comprehensive test script that verifies all functionality:

### JSON Output

```bash
$ ghtopdep-rs near/near-sdk-rs --format json --rows 2
{
  "dependents": [
    {
      "repo": "near/nearcore",
      "stars": "2.4K",
      "description": null
    },
    {
      "repo": "wormhole-foundation/wormhole",
      "stars": "1.7K",
      "description": null
    }
  ],
  "stats": {
    "total_repositories": 2510,
    "repositories_with_stars": 466,
    "elapsed_seconds": 27.35
  }
}
```