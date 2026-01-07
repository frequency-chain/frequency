# A tool to deploy AI associated schemas on chain


## Use to Deploy Schemas

### Setup

1. Pull the repository
1. Install dependencies `npm install`

## Usage

you can run one of the following commands based on the targeted chain.

```sh
npm run deploy:mainnet:intent
npm run deploy:mainnet:schema
```
or

```sh
npm run deploy:paseo
```
or

```sh
npm run deploy:local
```

The following environment variable allows you to change the default Alice sudo account used for deploying:

```sh
DEPLOY_SCHEMA_ACCOUNT_URI="//Alice"
```

e.g.

```sh
DEPLOY_SCHEMA_ACCOUNT_URI="//Bob" npm run deploy:paseo
```