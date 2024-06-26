cargo build --release
cp -r target/release/calcagebra-ls editors/vscode/out
cd editors/vscode
yarn
yarn package
cd ../../
mv editors/vscode/calcagebra* .
