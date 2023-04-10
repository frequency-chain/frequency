module.exports = {
    exit: true,
    extension: ["ts"],
    parallel: false,
    require:  ['ts-node/register','scaffolding/rootHooks.ts', 'scaffolding/extrinsicHelpers.ts'],
    spec: ["./{,!(node_modules|load-tests)/**}/*.test.ts"],
    timeout: 10000,
}

global.mochaConfig = module.exports;
