## kak-folder

A Rust-powered code folding plugin for Kakoune.

### Installation

Clone or download this repository.

Build and install the binary:
```
cargo install --path .
```

Install the plugin script:
```
cp plugin/kak-folder.kak "$KAK_CONFIG/autoload/"
```

### Usage

The plugin uses a `fold-map` user mode for all commands.

Keymaps
```
f - Create fold
x - Remove fold
m - Fold selection
e - Enable folding
d - Disable folding
r - Reset all folds
```
