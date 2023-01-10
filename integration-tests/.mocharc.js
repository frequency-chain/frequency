module.exports = {
    parallel: false,
    require:  ['scaffolding/rootHooks.ts', 'scaffolding/extrinsicHelpers.ts'],
    timeout: 500,
}

global.mochaConfig = module.exports;
