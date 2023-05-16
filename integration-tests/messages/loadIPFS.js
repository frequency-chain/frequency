
// Have to do this funky dynamic import stuff because IPFS is an ES module,
// but our integration tests are CommonJS. Some other dependency issues encountered
// in attempting to make the integration tests ES; may re-attempt at a later date.
exports.loadIpfs = async () => {
    const { create } = await import('ipfs-core');

    const node = await create();

    return node
}

exports.getBases = async () => {
    const { base64 } = await import('multiformats/bases/base64');
    const { base32 } = await import('multiformats/bases/base32');

    return { base64, base32 };
}
