
exports.loadIpfs = async () => {
    const { create } = await import('ipfs')

    const node = await create();

    return node
}
