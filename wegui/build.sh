#!/usr/bin/env bash -eu

CRATE_NAME="egui_demo_app"
 # NOTE: persistence use up about 400kB (10%) of the WASM!
FEATURES="glow,http,persistence,web_screen_reader"

OPEN=false
OPTIMIZE=false

while test $# -gt 0; do
  case "$1" in
    -h|--help)
      echo "build.sh [--optimize] [--open]"
      echo ""
      echo "  --optimize: Enable optimization step"
      echo "              Runs wasm-opt."
      echo "              NOTE: --optimize also removes debug symbols which are otherwise useful for in-browser profiling."
      echo ""
      echo "  --open:     Open the result in a browser"
      exit 0
      ;;

    -O|--optimize)
      shift
      OPTIMIZE=true
      ;;

    --open)
      shift
      OPEN=true
      ;;

    *)
      break
      ;;
  esac
done

rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.84

# This is required to enable the web_sys clipboard API which eframe web uses
# https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
# https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
export RUSTFLAGS=--cfg=web_sys_unstable_apis

# Clear output from old stuff:
rm -f "web/${CRATE_NAME}_bg.wasm"

echo "Building rust…"
BUILD=release

(
  cargo build \
    --release \
    --lib \
    --target wasm32-unknown-unknown \
    --no-default-features \
    --features ${FEATURES}
)

# Get the output directory (in the workspace it is in another location)
TARGET=`cargo metadata --format-version=1 | jq --raw-output .target_directory`

echo "Generating JS bindings for wasm…"
TARGET_NAME="${CRATE_NAME}.wasm"
WASM_PATH="${TARGET}/wasm32-unknown-unknown/$BUILD/$TARGET_NAME"
wasm-bindgen "${WASM_PATH}" --out-dir web --no-modules --no-typescript

# if this fails with "error: cannot import from modules (`env`) with `--no-modules`", you can use:
# wasm2wat target/wasm32-unknown-unknown/release/egui_demo_app.wasm | rg env
# wasm2wat target/wasm32-unknown-unknown/release/egui_demo_app.wasm | rg "call .now\b" -B 20 # What calls `$now` (often a culprit)

# to get wasm-strip:  apt/brew/dnf install wabt
# wasm-strip web/${CRATE_NAME}_bg.wasm

if [[ "${OPTIMIZE}" = true ]]; then
  echo "Optimizing wasm…"
  # to get wasm-opt:  apt/brew/dnf install binaryen
  wasm-opt "web/${CRATE_NAME}_bg.wasm" -O2 --fast-math -o "web/${CRATE_NAME}_bg.wasm" # add -g to get debug symbols
fi

echo "Finished web/${CRATE_NAME}_bg.wasm"

if [[ "${OPEN}" == true ]]; then
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux, ex: Fedora
    xdg-open http://localhost:8888/index.html
  elif [[ "$OSTYPE" == "msys" ]]; then
    # Windows
    start http://localhost:8888/index.html
  else
    # Darwin/MacOS, or something else
    open http://localhost:8888/index.html
  fi
fi
