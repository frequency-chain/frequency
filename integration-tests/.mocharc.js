module.exports = {
    parallel: false,
    require:  ['scaffolding/rootHooks.ts', 'scaffolding/extrinsicHelpers.ts'],
    timeout: 2000,
}

global.mochaConfig = module.exports;
