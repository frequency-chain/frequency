import '@frequency-chain/api-augment';
import assert from 'assert';
import { KeyringPair } from '@polkadot/keyring/types';
import { fileURLToPath } from 'url';
import path from 'path';

import { createAndFundKeypair, DOLLARS, generateValidProviderPayloadWithName } from '../scaffolding/helpers';
import { ExtrinsicHelper, EventMap } from '../scaffolding/extrinsicHelpers';
import { getFundingSource, getSudo } from '../scaffolding/funding';
import { isTestnet } from '../scaffolding/env';

// reconstruct __dirname in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let fundingSource: KeyringPair;

describe('Update Provider and Application', function () {
  let sudoKeys: KeyringPair;
  let providerKeys: KeyringPair;
  let providerId: bigint;
  let applicationId: bigint;

  before(async function () {
    sudoKeys = getSudo().keys;
    fundingSource = await getFundingSource(import.meta.url);
    providerKeys = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'upd-provider');

    // create MSA and register provider
    const f = ExtrinsicHelper.createMsa(providerKeys);
    await f.fundAndSend(fundingSource);

    const providerEntry = generateValidProviderPayloadWithName('UpdProv');
    const createProviderOp = ExtrinsicHelper.createProviderV2(providerKeys, providerEntry);
    const { target: providerEvent } = await createProviderOp.signAndSend();
    assert.notEqual(providerEvent, undefined, 'should emit ProviderCreated');
    providerId = providerEvent!.data.providerId.toBigInt();
    assert(providerId > 0n, 'providerId should be > 0');
    // create a base application to update
    const app = generateValidProviderPayloadWithName('BaseApp');
    const createAppOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, providerKeys, app);
    const { target: targetApp } = await createAppOp.sudoSignAndSend();
    assert.notEqual(targetApp, undefined, 'should emit ApplicationCreated');
    applicationId = targetApp!.data.applicationId.toBigInt();
    assert(applicationId >= 0n, 'applicationId should be > 0');
  });

  it('should successfully update provider defaultName via governance', async function () {
    if (isTestnet()) this.skip();
    const updated = generateValidProviderPayloadWithName('UpdProv2');
    const op = ExtrinsicHelper.updateProviderViaGovernance(sudoKeys, providerKeys, updated);
    const { target } = await op.sudoSignAndSend();
    assert.notEqual(target, undefined, 'should emit ProviderUpdated');

    // Verify default name changed via runtime API
    const ctx = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getProviderApplicationContext(
      providerId,
      null,
      null
    );
    assert.equal(ctx.isSome, true, 'provider context should be some');
    const result = ctx.unwrap();
    const defaultName = new TextDecoder().decode(result.defaultName.toU8a(true));
    assert.equal(defaultName, 'UpdProv2', 'provider default name should update');
  });

  it('should submit a proposal to update provider', async function () {
    if (isTestnet()) this.skip();
    const updated = generateValidProviderPayloadWithName('UpdProv3');
    const op = ExtrinsicHelper.proposeToUpdateProvider(providerKeys, updated);
    const { target } = await op.signAndSend();
    assert.notEqual(target, undefined, 'should emit Council.Proposed');
  });

  it('should update an existing application via governance', async function () {
    if (isTestnet()) this.skip();
    const updated = generateValidProviderPayloadWithName('AppUpdated');
    const op = ExtrinsicHelper.updateApplicationViaGovernance(sudoKeys, providerKeys, applicationId, updated);
    const { target } = await op.sudoSignAndSend();
    assert.notEqual(target, undefined, 'should emit ApplicationContextUpdated');

    // fetch context and verify name
    const ctx = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getProviderApplicationContext(
      providerId,
      applicationId,
      null
    );
    assert.equal(ctx.isSome, true, 'app context should be some');
    const result = ctx.unwrap();
    const defaultName = new TextDecoder().decode(result.defaultName.toU8a(true));
    assert.equal(defaultName, 'AppUpdated', 'app default name should update');
  });

  it('should submit a proposal to update application', async function () {
    if (isTestnet()) this.skip();
    // create an app to have an index
    const updated = generateValidProviderPayloadWithName('PropUpd');
    const op = ExtrinsicHelper.proposeToUpdateApplication(providerKeys, applicationId, updated);
    const { target: proposed } = await op.signAndSend();
    assert.notEqual(proposed, undefined, 'should emit Council.Proposed');
  });
});
