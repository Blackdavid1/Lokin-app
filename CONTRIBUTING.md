# Contributing to loki-app

## Prerequisites

- Rust toolchain (version pinned in `rust-toolchain.toml`, currently **1.91.0**)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli) for local deployment

## Building

```sh
cargo build --release --target wasm32-unknown-unknown
```

## Running tests

```sh
cargo test --features testutils
```

Or via the Makefile:

```sh
make test
```

## Editor setup (VS Code)

Install the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension, then open the workspace root. The `.vscode/settings.json` in this repo pre-configures:

| Setting | Value | Why |
|---|---|---|
| `rust-analyzer.cargo.features` | `["testutils"]` | Enables Soroban test utilities so `mock_all_auths` etc. resolve correctly |
| `rust-analyzer.check.command` | `clippy` | Matches CI lint step |
| `rust-analyzer.check.extraArgs` | `["--features", "testutils"]` | Keeps check consistent with test builds |

> **Note:** Do **not** set `rust-analyzer.cargo.target` to `wasm32-unknown-unknown`. The WASM target disables proc-macro analysis and breaks IDE completions. Tests are compiled for the host target; the `wasm32` target is only needed for `cargo build --release`.

### Other editors (LSP-based)

Pass `--features testutils` to `cargo check` in your LSP configuration. For example, in Neovim with `nvim-lspconfig`:

```lua
require("lspconfig").rust_analyzer.setup({
  settings = {
    ["rust-analyzer"] = {
      cargo = { features = { "testutils" } },
      check = {
        command = "clippy",
        extraArgs = { "--features", "testutils" },
      },
    },
  },
})
```

## Code style

- Editor defaults are defined in [`.editorconfig`](.editorconfig) (Rust: 4 spaces, LF line endings, UTF-8, final newline). Configure your editor to respect it to avoid format noise.
- `cargo fmt` is enforced in CI (`make fmt-check`)
- `cargo clippy --features testutils -- -D warnings` must pass
- Keep contract functions documented with a single-line comment explaining the intent, not the implementation
