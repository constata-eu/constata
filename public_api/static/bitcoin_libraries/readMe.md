run the following commands:
- npm install
- browserify -r miscreant -r tiny-secp256k1 -r bip32 -r bip39 -r bitcoinjs-lib -r bitcoinjs-message -r buffer -r request -r ecpair -r countrily -r translate ./src/main.js -o ./dist/bundle.js