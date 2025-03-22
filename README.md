# rust-gui-experiments <img width=20 height=20 src="www/favicon.png"></img>

<img width=100 height=100 src="https://lap.dev/images/floem.svg"></img>
<img src="https://raw.githubusercontent.com/iced-rs/iced/refs/heads/master/docs/logo.svg" width="100px"></img>
![#MadeWithSlint](https://raw.githubusercontent.com/slint-ui/slint/refs/heads/master/logo/MadeWithSlint-logo-light.svg#gh-light-mode-only)
![#MadeWithSlint](https://raw.githubusercontent.com/slint-ui/slint/refs/heads/master/logo/MadeWithSlint-logo-dark.svg#gh-dark-mode-only)

Trying out different GUI crates, just to get to know what is possible and how to do it.

As a sample task Sudoku game is implemented with different feature-gated UIs:
* [floem](https://github.com/lapce/floem)
* [iced](https://github.com/iced-rs/iced)
* [slint](https://github.com/slint-ui/slint)
* [wasm](https://github.com/rustwasm/wasm-bindgen)

Other examples for wasm are [wasm-bindgen's](https://rustwasm.github.io/wasm-bindgen/examples) and 
[fosskers's](https://www.fosskers.ca/en/demo/game-of-life) 
([repo](https://github.com/fosskers/fosskers.ca/tree/master/rust/game-of-life))

I planned to try doing in with [Tauri](https://tauri.app/), but it
appeared too complicated for me to get on the first try.

## running most versions

Each GUI is gated behind a feature with the same name:

```shell
cargo run --features floem
cargo run --features iced
cargo run --features slint
```

## running `wasm` version

```shell
wasm-pack build --target web --features wasm
miniserve . --index "index.html" -p 8080
```