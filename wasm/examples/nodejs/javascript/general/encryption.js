const lmt = require('../../../../nodejs/lmt');

lmt.initConsolePanicHook();

(async () => {

    let encrypted = lmt.encryptXChaCha20Poly1305("my message", "my_password");
    console.log("encrypted:", encrypted);
    let decrypted = lmt.decryptXChaCha20Poly1305(encrypted, "my_password");
    console.log("decrypted:", decrypted);

})();
