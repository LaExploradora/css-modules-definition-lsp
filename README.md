# Go to definition from TSX to SCSS Modules

This extension attempts to replicate the VS Code pattern, where Ctrl+click on a classname from an imported SCSS module takes you to definition of that class.

Ctrl/Cmd+click on `styles.button` in a `.tsx` file jumps to `.button` in `Button.module.scss`.

The extension does not need any change in your project: no TypeScript plugin, no `tsconfig` change, no `package.json` change.

## Important disclaimer

This is simply a small and fun research project I've made while attempting to learn Rust and migrating from VS Code to Zed.

This extension is provided as-is and is not guaranteed to work, given my low level of expertise at the moment, but it works on my machine™.

Any bug reports or GH Issues are welcome, but I cannot guarantee that they will be resolved.

## How it works

A Zed extension cannot add UI or hook clicks. This extension only tells Zed to start a language server:

```
Ctrl+click styles.button in .tsx
   -> Zed sends LSP textDocument/definition
   -> cssmodules-language-server finds .button in the style file
   -> Zed opens the file at that line
```

- The server is the npm package [`cssmodules-language-server`](https://github.com/antonk52/cssmodules-language-server).
- Zed installs it into the extension's own work folder, never into your project.
- The server version is pinned (`PINNED_VERSION` in `src/lib.rs`), so upstream releases never change the extension by themselves. To update it, change the constant, run the checks below and the manual checklist, then release a new extension version.
- Zed's Node.js runs it.
- `initializationOptions.camelCase` is `"dashes"`, so `.btn-icon` matches `styles.btnIcon`.

## Supported

- Style files: `.css`, `.scss`, `.sass`, `.less`, `.styl`.
- Languages: TSX, TypeScript, JavaScript.
- Relative imports: `import styles from './Button.module.scss'`.
- Alias imports, when `paths` or `baseUrl` is in `tsconfig.json` or `jsconfig.json` (the server follows `extends`).
- Bracket access: `styles['btn-icon']`.

## Known issues

| Issue | Detail |
|---|---|
| **Two results on Ctrl+click** | `vtsls` also answers go-to-definition. Vite declares `*.module.scss` in `vite/client.d.ts`, so Zed shows that file next to the real style file. Pick the style file from the list. Zed has no setting to turn off one capability of one server. |
| `cx('btn')` from `classnames/bind` | Not supported. The server only matches `object.field` and `object['field']`. This comes from reading the server source. It is not tested in Zed. |
| Aliases outside `tsconfig.json` / `jsconfig.json` | Not supported, for example aliases only in `vite.config.ts` or the webpack config. |
| `tsconfig.json` with `references` only | Probably not supported. The server does not follow `references`, so `paths` kept in `tsconfig.app.json` is not found. This comes from reading the server source. It is not tested in Zed. |
| `css-modules-kit` installed at the same time | Both extensions answer for the same files. Disable one while you test. |

## Development

You need Rust from `rustup`. Zed adds the `wasm32-wasip2` target itself.

```sh
cargo test                              # unit tests and manifest tests
cargo check --target wasm32-wasip2      # compile as Zed does
```

To try the extension in Zed: command palette -> `zed: install dev extension` -> select this folder.
To see logs: command palette -> `zed: open log`.

## Credits

- [`cssmodules-language-server`](https://github.com/antonk52/cssmodules-language-server) by [antonk52](https://github.com/antonk52). This extension only installs and starts this server. All go-to-definition logic belongs to it.
- [`css-modules-kit`](https://github.com/mizdra/css-modules-kit) by [mizdra](https://github.com/mizdra). It is the other approach to CSS Modules support in Zed: a TypeScript plugin that needs project changes. It was the main reference for this project, and it is the better choice if you accept those changes.
