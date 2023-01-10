
export namespace env {
    export const providerUrl = process.env.WS_PROVIDER_URL;
    export const verbose = (process.env.VERBOSE_TESTS === 'true' || process.env.VERBOSE_TESTS === '1');
}
