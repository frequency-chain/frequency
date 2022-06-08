# CONTRIBUTING

For contributing guidelines see the [Project Liberty Contributing Guidelines](https://github.com/LibertyDSNP/meta/blob/main/CONTRIBUTING.md).

## Useful Links

- [Type Definitions](https://github.com/polkadot-js/api/blob/master/packages/types/src/types/definitions.ts)

## How to Release

1. Create a New Release on GitHub.com
2. Set tag to `js-mrc-rpc-v[X.X.X]` following [Semver 2.0](https://semver.org/)
3. Set title to "js-mrc-rpc-v[version] Major Feature Name"
4. Set contents to follow [KeepAChangeLog.com 1.0](https://keepachangelog.com/en/1.0.0/), but limited to just the new release information
    ```markdown
    ## [0.1.0] - 2017-06-20
    ### Added
    - New thing
    ### Changed
    - Different thing
    ### Removed
    - Not a thing anymore
    ```
5. Publish
6. CI will build and publish to the npm repository
