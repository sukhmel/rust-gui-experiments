# rust-gui-experiments <img width=20 height=20 src="www/favicon.png"></img>

<img width=100 height=100 src="https://lap.dev/images/floem.svg"></img>
<img width="100px" src="https://raw.githubusercontent.com/iced-rs/iced/refs/heads/master/docs/logo.svg"></img>
<img width="100px" src="./logos/WebAssembly.svg"></img>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="./logos/zed-logo-light.svg">
  <source media="(prefers-color-scheme: dark)" srcset="./logos/zed-logo-dark.svg">
  <img alt="gpui by Zed Industries" src="./logos/zed-logo-dark.svg">
</picture>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/slint-ui/slint/refs/heads/master/logo/MadeWithSlint-logo-light.svg">
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/slint-ui/slint/refs/heads/master/logo/MadeWithSlint-logo-dark.svg">
  <img alt="#MadeWithSlint" src="https://raw.githubusercontent.com/slint-ui/slint/refs/heads/master/logo/MadeWithSlint-logo-dark.svg">
</picture>
<picture>
  <source media="(prefers-color-scheme: light)" srcset="https://ratatui.rs/_astro/hero-light.DXG5UAQy_1YWiFp.webp">
  <source media="(prefers-color-scheme: dark)" srcset="https://ratatui.rs/_astro/hero-dark.sdDaGsSQ_Z1JuwEh.webp">
  <img width="100px" alt="ratatui" src="https://ratatui.rs/_astro/hero-dark.sdDaGsSQ_Z1JuwEh.webp">
</picture>


Trying out different GUI crates, just to get to know what is possible and how to do it.

As a sample task Sudoku game is implemented with different feature-gated UIs:
* [x] [floem](https://github.com/lapce/floem)
* [x] [iced](https://github.com/iced-rs/iced)
* [x] [slint](https://github.com/slint-ui/slint)
* [x] [wasm](https://github.com/rustwasm/wasm-bindgen)
* [x] [egui](https://github.com/emilk/egui)
* [x] [gpui](https://github.com/zed-industries/zed/tree/main/crates/gpui)
* [ ] [xilem](https://github.com/linebender/xilem)
* [ ] [leptos](https://github.com/leptos-rs/leptos)
* [ ] [rui](https://github.com/audulus/rui)
* [x] [ratatui](https://github.com/ratatui/ratatui)
* [ ] [kas](https://github.com/kas-gui/kas)
* [ ] (?) [Tauri](https://tauri.app/)
* [ ] (?) [GTK](https://github.com/gtk-rs/gtk4-rs) bindings 
* [ ] (?) [Qt](https://github.com/KDAB/cxx-qt/) bindings?

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
cargo run --features egui
cargo run --features gpui
cargo run --features xilem
cargo run --features leptos
cargo run --features rui
cargo run --features ratatui
cargo run --features kas
```

## GPUI on macOS

GPUI will require metal to be installed, so you will need full Xcode app (not just command line tools). Then likely:

```shell
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
sudo xcodebuild -license
xcodebuild -runFirstLaunch
xcodebuild -downloadComponent MetalToolchain
```

## running `wasm` version

```shell
wasm-pack build --target web --features wasm
miniserve . --index "index.html" -p 8080
```
