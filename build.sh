rm -rf *.vsix
cargo build --release
cp -r target/release/calcagebra-ls editors/vscode/out
cd editors/vscode
yarn install
yarn package