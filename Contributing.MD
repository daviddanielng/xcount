# Contributing to XCount

Thanks for taking the time to contribute. Here is everything you need to get started.

---

## Reporting Issues

If you run into a bug or unexpected behaviour, open an issue on [GitHub](https://github.com/daviddanielng/xcount/issues) and include:

- Your OS and architecture
- The command you ran
- What you expected vs. what happened
- Any output or error messages

---

## Submitting a Pull Request

1. Fork the repository and create a branch from `main`:

   ```bash
   git checkout -b fix/your-fix-name
   ```

2. Make your changes.

3. Format and lint before committing:

   ```bash
   cargo fmt
   cargo clippy
   ```

4. Keep changes focused — one fix or feature per PR.

5. If your change affects behaviour or adds a flag, update the README to reflect it.

6. Open a pull request against `main` with a clear description of what was changed and why.

---

## Building Locally

```bash
git clone https://github.com/daviddanielng/xcount.git
cd xcount
cargo build --release
```

---

## License

By contributing, you agree that your changes will be licensed under the [MIT License](./LICENSE).