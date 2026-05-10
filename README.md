<div align="center">

![Oak](oak.png)

# Oak

A modern, fast, and beautiful `tree` command.

</div>

---

## Why Oak?

The standard `tree` command hasn't evolved in decades. Oak is a ground-up rewrite that makes directory listing actually enjoyable.

- **Git-aware by default** — respects `.gitignore` automatically (no more `node_modules` spam)
- **Rich icons** — macOS-friendly Unicode icons by default, with Nerd Font icons available
- **Smart sorting** — sort by name, size, extension, or modification time
- **Clean, colorful output** — modern terminal colors that respect `NO_COLOR`
- **Fast** — built in Rust with parallel gitignore filtering

## Installation

```bash
cargo install --path .
```

Or copy the binary directly:

```bash
cargo build --release
cp target/release/oak ~/.local/bin/
```

## Usage

```bash
oak                          # current directory
oak -st                      # with sizes and timestamps
oak -L 2                     # max depth 2
oak -a                       # show hidden files
oak -S name                  # sort alphabetically
oak -P '\.rs$'              # filter by regex
```

## Sorting

| Flag | Description |
|------|-------------|
| `-S mtime` | **Default** — most recently modified first |
| `-S name` | Alphabetically |
| `-S size` | Largest first |
| `-S ext` | By file extension |

## Options

```
-L, --level <LEVEL>      Maximum display depth
-a, --all                Show hidden files
-s, --sizes              Show file sizes
-t, --times              Show modification times
-P, --pattern <PATTERN>  Only show files matching regex
-I, --exclude <EXCLUDE>  Exclude files matching regex
    --no-ignore          Don't respect .gitignore
    --no-icons           Disable icons
    --icon-style <STYLE> Icon style (unicode, nerd-font)
    --no-color           Plain text output
    --dirs-only          Show directories only
    --files-only         Show files only
-S, --sort <SORT>        Sort order (mtime, name, size, ext)
```

## License

MIT OR Apache-2.0. See [LICENSE](LICENSE).
