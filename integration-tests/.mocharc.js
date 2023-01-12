module.exports = {
    exit: true,
    extension: ["ts"],
    parallel: false,
    require:  ['ts-node/register','scaffolding/rootHooks.ts', 'scaffolding/extrinsicHelpers.ts'],
    spec: ["./{,!(node_modules)/**}/*.test.ts"],
    timeout: 500,
}

global.mochaConfig = module.exports;
