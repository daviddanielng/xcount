# XCount

A CLI tool for extracting follower and following counts for X (Twitter) accounts via direct HTTP requests — no browser required.

**GitHub:** [github.com/daviddanielng/xcount](https://github.com/daviddanielng/xcount)

---

## Requirements

- Linux: pre-compiled binary available
- Windows / macOS: must build from source (see [Building from Source](#building-from-source))

---

## Installation

### Linux (pre-compiled binary)

Download the latest binary from the [Releases](https://github.com/daviddanielng/xcount/releases) tab on GitHub, then make it executable:

```bash
chmod +x xcount
./xcount --help
```

Optionally move it onto your `$PATH`:

```bash
mv xcount ~/.local/bin/xcount
```

### Building from Source

Requires [Rust](https://rustup.rs) (stable toolchain).

```bash
git clone https://github.com/daviddanielng/xcount.git
cd xcount
cargo build --release
```

The compiled binary will be at `target/release/xcount`. You can move it anywhere on your `$PATH`:

```bash
cp target/release/xcount ~/.local/bin/xcount   # Linux / macOS
```

---

## Usage

```
xcount [OPTIONS] --username <USERNAMES> | --input <FILE>
```

### Options

| Flag | Short | Default | Description |
|---|---|---|---|
| `--username <USERNAMES>` | `-u` | — | Comma-separated list of usernames *(mutually exclusive with `-i`)* |
| `--input <FILE>` | `-i` | — | Path to a file with one username per line *(mutually exclusive with `-u`)* |
| `--format <FORMAT>` | `-f` | `json` | Output format: `json`, `csv`, or `excel` |
| `--output <DIR>` | `-o` | `.` | Directory to write the output file into |
| `--delay <SECONDS>` | `-d` | `1` | Seconds to wait between each request |
| `--verbose` | `-v` | `false` | Print progress and log info to stderr |

---

### Fetch one or more usernames

```bash
xcount -u daviddanielng
xcount -u daviddanielng,elonmusk,github
```

Results are always saved to a file — never printed. The file is named `output-{timestamp}.{ext}` and written to the current directory by default.

```
output-{yyyy-mm-dd-hh::mm}.json
```

---

### Read usernames from a file

The file must have one username per line.

```bash
xcount -i ./users.txt
xcount -i /home/daniel/usernames.txt
```

`users.txt` example:
```
daviddanielng
elonmusk
github
```

---

### Choose an output format

Use `-f` to set the format. Accepted values are `json`, `csv`, and `excel`. Defaults to `json`.

```bash
xcount -u daviddanielng,elonmusk -f csv
xcount -u daviddanielng,elonmusk -f excel
xcount -i ./users.txt -f json
```

**Output filenames follow the pattern `output-{timestamp}.{ext}`:**
```
output-{yyyy-mm-dd-hh::mm}.json
output-{yyyy-mm-dd-hh::mm}.csv
output-{yyyy-mm-dd-hh::mm}.xlsx
```

---

### Choose an output directory

Use `-o` to set the directory the output file is written to. Defaults to the current directory.

```bash
xcount -u daviddanielng,elonmusk -o ./results
xcount -i ./users.txt -f excel -o /tmp
```

---

### Delay between requests

Use `-d` to add a pause (in seconds) between each username fetch. Defaults to `1`. Increase this to reduce the chance of getting IP-banned when processing large lists.

```bash
xcount -u daviddanielng,elonmusk,github -d 3
xcount -i ./users.txt -f csv -d 5
```

---

### Verbose output

Add `-v` to print progress logs during the run. Useful for monitoring or debugging.

```bash
xcount -u daviddanielng,elonmusk -v
```

```
[INFO] Fetching profile: daviddanielng
[INFO] Fetching profile: elonmusk
[INFO] Done. Output saved to output-{yyyy-mm-dd-hh::mm}.json
```

---

## Error Handling

If a username cannot be fetched (not found, private, rate-limited, etc.) it is silently skipped and will not appear in the output. It is left to you to verify the output and confirm all expected usernames are present.

---

## Output

Each record in the output contains the username, follower count, following count, and tweet count.

**JSON example:**
```json
[
  {
    "username": "daviddanielng",
    "followers": 166,
    "following": 418,
    "tweets": 6076
  }
]
```

> **Note on accuracy:** Counts are read directly from the page's `application/ld+json` structured data, not parsed from the shortened display text (e.g. `1.2M`, `418K`) shown in the UI. This means counts are exact, even for accounts with very large follower numbers.

---

## Notes

- `-u` and `-i` are mutually exclusive — exactly one must be provided.
- Output is **always written to a file** — results are never printed to the terminal.
- The output file is named `output-{timestamp}.{ext}` and written to the directory set by `-o` (defaults to the current directory).
- The delay set with `-d` applies between usernames, not before the first one.
- The pre-compiled binary targets **Linux x64** only. All other platforms require building from source with `cargo`.

---

## Contributing

Issues and pull requests are welcome. If you run into a bug or want to suggest a feature, open an issue on the [GitHub repository](https://github.com/daviddanielng/xcount/issues).

When submitting a PR:
- Run `cargo fmt` and `cargo clippy` before committing
- Keep changes focused — one fix or feature per PR
- If you're adding behaviour, update this README to reflect it

---

## License

MIT © [David Daniel](https://github.com/daviddanielng)
