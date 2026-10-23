# Contributing to ValoShell

Thanks for helping. ValoShell is young, so bug reports are as valuable as code.
Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting bugs and ideas

Use the [issue forms](https://github.com/Soulhackzlol/ValoShell/issues/new/choose). A screenshot
and the steps to reproduce make a bug far easier to fix. Report security problems privately
through [a security advisory](https://github.com/Soulhackzlol/ValoShell/security/advisories/new).

## Setting up

You need Windows 10/11, [Rust](https://rustup.rs) 1.96+ and [Node.js](https://nodejs.org) 22+.

```sh
npm install
npm run ui:dev    # the menu in a browser with demo data; no VALORANT needed
npm run dev       # the full app, talking to your running Riot Client
```

Most UI work, including new styles, can be done entirely in `npm run ui:dev`.

## Before opening a pull request

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace    # also regenerates ui/src/bindings; commit any changes
npm run ui:check
npm run ui:test
```

CI runs the same checks on every pull request.

## Code style

- Small functions (40 lines at most) with clear, unabbreviated names.
- Comments explain why, not what.
- `crates/domain` stays free of I/O; network calls live in `crates/riot`.
- Keep pull requests focused on one change.

## Adding a menu style

Styles live entirely in the UI; no Rust changes are needed.

1. Add its id to `Skin` in `ui/src/studio/types.ts`.
2. Add its bundle (colours, font, corners, backdrop) to `SKINS` in `ui/src/studio/skins.ts`.
3. Add its pieces and CSS modules in `ui/src/pieces/styled/`, and its layouts in `skins.ts`.
4. Check every page in `npm run ui:dev` and add screenshots to the pull request.

## What ValoShell won't accept

ValoShell only does what the official client can do through the Riot Client's local API.
Changes that automate gameplay or agent select, read game memory, or touch the game process
won't be merged.

## License

By contributing you agree that your work is released under the [GPL-3.0-or-later](LICENSE).
