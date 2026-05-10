<div align="center">

![Oak](assets/oak.png)

# Oak

A modern, fast, and beautiful `tree` command.

</div>

---

## Why Oak?

The standard `tree` command hasn't evolved in decades. Oak is a ground-up rewrite that makes directory listing actually enjoyable.

- **Git-aware by default** — respects `.gitignore` automatically (no more `node_modules` spam)
- **Rich icons** — macOS-friendly Unicode icons
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
oak -L 2 --no-icons --save-config
```

## Configuration

Save your preferred options once:

```bash
oak -L 2 -S name --no-icons --save-config
```

Oak writes defaults to:

```text
$XDG_CONFIG_HOME/oak/config
```

If `XDG_CONFIG_HOME` is not set, Oak uses:

```text
~/.config/oak/config
```

Future `oak` runs automatically use the saved defaults. Any flag you pass on a later command overrides the saved value for that setting. Pass `--no-config` to ignore the saved config for one command.

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
    --hide-hidden        Hide hidden files
-s, --sizes              Show file sizes
    --no-sizes           Hide file sizes
-t, --times              Show modification times
    --no-times           Hide modification times
-P, --pattern <PATTERN>  Only show files matching regex
-I, --exclude <EXCLUDE>  Exclude files matching regex
    --save-config        Save these options as future defaults and exit
    --no-config          Do not read saved config
    --no-ignore          Don't respect .gitignore
    --ignore             Respect .gitignore
    --no-icons           Disable icons
    --icons              Enable icons
    --no-color           Plain text output
    --color              Enable color output
    --dirs-only          Show directories only
    --files-only         Show files only
-S, --sort <SORT>        Sort order (mtime, name, size, ext)
```

## License

MIT - See [LICENSE](LICENSE).
