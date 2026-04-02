# LMT WASM SDK

An integration wrapper around [`lmt-wasm`](https://www.npmjs.com/package/lmt-wasm) module that uses [`websocket`](https://www.npmjs.com/package/websocket) W3C adaptor for WebSocket communication.

This is a Node.js module that provides bindings to the LMT WASM SDK strictly for use in the Node.js environment. The web browser version of the SDK is available as part of official SDK releases at [https://github.com/cadaritre/Lapis-monetae/releases](https://github.com/cadaritre/Lapis-monetae/releases)

## Usage

LMT NPM module exports include all WASM32 bindings.
```javascript
const LMT = require('lmt');
console.log(lmt.version());
```

## Documentation

Documentation is available at [https://lmt.aspectron.org/docs/](https://lmt.aspectron.org/docs/)


## Building from source & Examples

SDK examples as well as information on building the project from source can be found at [https://github.com/cadaritre/Lapis-monetae/tree/master/wasm](https://github.com/cadaritre/Lapis-monetae/tree/master/wasm)

## Releases

Official releases as well as releases for Web Browsers are available at [https://github.com/cadaritre/Lapis-monetae/releases](https://github.com/cadaritre/Lapis-monetae/releases).

Nightly / developer builds are available at: [https://aspectron.org/en/projects/lmt-wasm.html](https://aspectron.org/en/projects/lmt-wasm.html)

