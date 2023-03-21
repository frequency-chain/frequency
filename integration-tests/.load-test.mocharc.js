module.exports = {
    exit: true,
    extension: ["ts"],
    parallel: false,
    require:  ['ts-node/register','scaffolding/rootHooks.ts', 'scaffolding/extrinsicHelpers.ts'],
    spec: ["./load-tests/**/*.test.ts"],
    timeout: 60_000,
}

global.mochaConfig = module.exports;
