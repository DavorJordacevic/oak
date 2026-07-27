# Oak Cheat Sheet

## Quick Start

```bash
oak                  # list current directory
oak /path/to/dir     # list specific directory
oak -L 2             # max depth 2
```

## Essential Combos

```bash
oak -st              # sizes + timestamps
oak -L 2 -st         # sizes + timestamps, depth 2
oak -a -L 3          # hidden files, depth 3
oak -s -t --du       # sizes, timestamps, dir rollups
```

## Find & Filter

```bash
oak --find main      # fuzzy search filenames
oak --find-text "TODO"   # search file contents
oak -P '\.rs$'       # regex: only .rs files
oak -I 'node_modules'    # regex: exclude pattern
oak -P 'src/.*\.rs' -I 'test'    # include + exclude
```

## Sort Options

```bash
oak -S name          # sort alphabetically
oak -S size          # sort by size (largest first)
oak -S mtime         # sort by mod time (default)
oak -S ext           # sort by extension
```

## Export Formats

```bash
oak --json           # JSON export
oak --csv            # CSV export
oak --graph          # Graphviz DOT (pipe to `dot -Tpng`)
oak --md             # Markdown list
oak --html           # HTML nested list
```

## Display Toggles

```bash
oak --dirs-only      # only directories
oak --files-only     # only files
oak --no-icons       # disable icons
oak --no-color       # disable color
oak --timeline       # group by modification recency
oak --clip           # copy output to clipboard
```

## Context Toggles

```bash
oak --perms          # show permissions
oak --git            # show git status
oak --git-blame      # show last committer
oak --links          # show symlink targets
oak --du             # show directory sizes
oak --prune          # prune empty dirs after filter
oak --stats          # show statistics footer
```

Disable context (`--no-*`):

```bash
oak --no-sizes --no-times --no-icons --no-color --no-du --no-git --no-git-blame --no-perms --no-links --no-stats
```

## Ignore Control

```bash
oak --no-ignore      # ignore .gitignore / .ignore
oak --ignore         # respect .gitignore (default)
oak -a               # show hidden (dotfiles)
oak --hide-hidden    # hide hidden files (default)
```

## Config

```bash
oak -L 2 -st --save-config    # save current flags as defaults
oak --no-config               # bypass saved config for this run
# Config file: ~/.config/oak/config
```

## Piping & Scripts

```bash
oak --json | jq '.'                # pipe JSON
oak --csv | column -t -s,          # pipe CSV
oak --graph | dot -Tpng > tree.png  # render graph
oak --md > tree.md                  # save markdown
oak --html > tree.html              # save HTML
oak --find-text "TODO" -S name      # search + sort
oak --no-config --json -P '\.rs$'   # script-friendly
```

## Full Context (typical defaults)

```bash
oak -L 3 -st --du --perms --git --links --prune
```
