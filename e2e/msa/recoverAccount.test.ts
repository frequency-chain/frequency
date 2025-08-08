import '@frequency-chain/api-augment';
import '@frequency-chain/recovery-sdk';
import {
  ContactType,
  generateRecoverySecret,
  getRecoveryCommitment,
  getIntermediaryHashes,
} from '@frequency-chain/recovery-sdk';
import assert from 'assert';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper, RecoveryCommitmentPayload, AddKeyData } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {
  createAndFundKeypairs,
  signPayloadSr25519,
  DOLLARS,
  generateRecoveryCommitmentPayload,
  generateSignedAddKeyProofWithType,
  assertEvent,
  createKeys,
  getEthereumKeyPairFromUnifiedAddress,
  generateAddKeyPayload,
  generateValidProviderPayloadWithName,
} from '../scaffolding/helpers';
import { u64 } from '@polkadot/types';
import { getUnifiedAddress } from '@frequency-chain/ethereum-utils';
import { u8aToHex } from '@polkadot/util';

let fundingSource: KeyringPair;

describe('Account Recovery Testing', function () {
  // Common variables used across tests
  let recoveryProviderKey: KeyringPair;
  let recoveryProviderMsaId: u64;
  const testEmail = 'test@example.com';

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    recoveryProviderKey = fundingSource;

    // Setup recovery provider MSA (this can be shared across tests)
    [recoveryProviderKey] = await createAndFundKeypairs(fundingSource, ['recoveryProviderKeys'], 2n * DOLLARS);

    const { target: providerMsaTarget } = await ExtrinsicHelper.createMsa(recoveryProviderKey).signAndSend();
    assert.notEqual(providerMsaTarget?.data.msaId, undefined, 'Provider MSA Id not in expected event');
    recoveryProviderMsaId = providerMsaTarget!.data.msaId;
    const providerEntry = generateValidProviderPayloadWithName('RecoveryProvider');

    // Register the MSA as a provider first (required before approval)
    const createProviderOp = ExtrinsicHelper.createProvider(recoveryProviderKey, providerEntry);
    const { eventMap: providerEventMap } = await createProviderOp.fundAndSend(fundingSource);
    assertEvent(providerEventMap, 'system.ExtrinsicSuccess');
    assertEvent(providerEventMap, 'msa.ProviderCreated');

    // Approve the recovery provider (direct call for development environment)
    const approveProviderOp = ExtrinsicHelper.approveRecoveryProvider(fundingSource, recoveryProviderKey);
    const { eventMap: approveEventMap } = await approveProviderOp.fundAndSend(fundingSource);
    assertEvent(approveEventMap, 'system.ExtrinsicSuccess');
    assertEvent(approveEventMap, 'msa.RecoveryProviderApproved');
  });

  // Helper function to set up a fresh user with recovery commitment for each test
  async function setupUserWithRecoveryCommitment() {
    // Create new keys for each test to avoid state conflicts
    const [testUserKeys, testNewControlKeys] = await createAndFundKeypairs(
      fundingSource,
      [`testUser-${Date.now()}`, `testNewControl-${Date.now()}`],
      2n * DOLLARS
    );

    // Setup user MSA
    const { target: userMsaTarget } = await ExtrinsicHelper.createMsa(testUserKeys).signAndSend();
    assert.notEqual(userMsaTarget?.data.msaId, undefined, 'User MSA Id not in expected event');
    const testMsaId = userMsaTarget!.data.msaId;

    // Generate fresh recovery secret and commitment for this test
    const testRecoverySecret = generateRecoverySecret();
    const testRecoveryCommitment = getRecoveryCommitment(testRecoverySecret, ContactType.EMAIL, testEmail);

    const testRecoveryCommitmentData: RecoveryCommitmentPayload = {
      discriminant: 'RecoveryCommitmentPayload',
      recoveryCommitment: testRecoveryCommitment,
    };
    const testPayload = await generateRecoveryCommitmentPayload({ ...testRecoveryCommitmentData });
    const testCodecForPayload = ExtrinsicHelper.api.registry.createType(
      'PalletMsaRecoveryCommitmentPayload',
      testPayload
    );
    const testOwnerSig = signPayloadSr25519(testUserKeys, testCodecForPayload);

    const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(testUserKeys, testOwnerSig, testPayload);
    const { eventMap: commitmentEventMap } = await addRecoveryCommitmentOp.fundAndSend(fundingSource, false);
    assertEvent(commitmentEventMap, 'system.ExtrinsicSuccess');
    assertEvent(commitmentEventMap, 'msa.RecoveryCommitmentAdded');

    return {
      userKeys: testUserKeys,
      newControlKeys: testNewControlKeys,
      msaId: testMsaId,
      recoverySecret: testRecoverySecret,
      recoveryCommitment: testRecoveryCommitment,
    };
  }

  describe('recoverAccount', function () {
    it('should successfully recover account with Sr25519 signed new control key', async function () {
      // Set up fresh user and recovery commitment for this test
      const { userKeys, newControlKeys, msaId, recoverySecret } = await setupUserWithRecoveryCommitment();

      // Generate intermediary hashes
      const { a: intermediaryHashA, b: intermediaryHashB } = getIntermediaryHashes(
        recoverySecret,
        ContactType.EMAIL,
        testEmail
      );

      // Create add key payload for the new control key
      const addKeyData: AddKeyData = {
        msaId: msaId,
        newPublicKey: getUnifiedAddress(newControlKeys),
      };
      const { payload: addKeyPayload, signature: newControlKeyProof } = await generateSignedAddKeyProofWithType(
        addKeyData,
        newControlKeys
      );

      // Execute recovery
      const recoverAccountOp = ExtrinsicHelper.recoverAccount(
        recoveryProviderKey,
        intermediaryHashA,
        intermediaryHashB,
        newControlKeyProof,
        addKeyPayload
      );

      const { eventMap } = await recoverAccountOp.fundAndSend(fundingSource, false);

      // Assert success events
      assertEvent(eventMap, 'system.ExtrinsicSuccess');
      assertEvent(eventMap, 'msa.AccountRecovered');
      assertEvent(eventMap, 'msa.RecoveryCommitmentInvalidated');

      // Verify the AccountRecovered event contains expected data
      const accountRecoveredEvent = eventMap['msa.AccountRecovered'];
      assert.notEqual(accountRecoveredEvent, undefined, 'AccountRecovered event should be present');
      assert.equal(accountRecoveredEvent.data[0].toString(), msaId.toString(), 'MSA ID should match');
      assert.equal(
        accountRecoveredEvent.data[1].toString(),
        recoveryProviderMsaId.toString(),
        'Recovery provider MSA ID should match'
      );
      assert.equal(
        accountRecoveredEvent.data[2].toString(),
        getUnifiedAddress(newControlKeys),
        'New control key should match'
      );
    });

    it('should successfully recover account with EIP-712 signed new control key', async function () {
      const { userKeys, msaId, recoverySecret } = await setupUserWithRecoveryCommitment();

      const { a: intermediaryHashA, b: intermediaryHashB } = getIntermediaryHashes(
        recoverySecret,
        ContactType.EMAIL,
        testEmail
      );

      const newEthereumControlKey = createKeys('newEthereumControlKey', 'ethereum');

      // Create add key payload for the new Ethereum control key and sign with EIP-712 by checking type === 'ethereum'
      const addKeyData: AddKeyData = {
        msaId: msaId,
        newPublicKey: getUnifiedAddress(newEthereumControlKey),
      };
      const { payload: addKeyPayload, signature: newControlKeyProof } = await generateSignedAddKeyProofWithType(
        addKeyData,
        newEthereumControlKey
      );

      // Execute recovery
      const recoverAccountOp = ExtrinsicHelper.recoverAccount(
        recoveryProviderKey,
        intermediaryHashA,
        intermediaryHashB,
        newControlKeyProof,
        addKeyPayload
      );

      const { eventMap } = await recoverAccountOp.fundAndSend(fundingSource, false);

      // Assert success events
      assertEvent(eventMap, 'system.ExtrinsicSuccess');
      assertEvent(eventMap, 'msa.AccountRecovered');
      assertEvent(eventMap, 'msa.RecoveryCommitmentInvalidated');

      // Verify the AccountRecovered event contains expected data
      const accountRecoveredEvent = eventMap['msa.AccountRecovered'];
      assert.notEqual(accountRecoveredEvent, undefined, 'AccountRecovered event should be present');
      assert.equal(accountRecoveredEvent.data[0].toString(), msaId.toString(), 'MSA ID should match');
      assert.equal(
        accountRecoveredEvent.data[1].toString(),
        recoveryProviderMsaId.toString(),
        'Recovery provider MSA ID should match'
      );
      assert.equal(
        accountRecoveredEvent.data[2].toString(),
        getUnifiedAddress(newEthereumControlKey),
        'New Ethereum control key should match'
      );
    });

    it('should fail when called by non-registered provider', async function () {
      // Set up fresh user and recovery commitment for this test
      const { newControlKeys, msaId, recoverySecret } = await setupUserWithRecoveryCommitment();

      // Create a non-approved provider
      const [nonApprovedProvider] = await createAndFundKeypairs(fundingSource, ['nonApprovedProvider'], 2n * DOLLARS);

      // Create MSA for non-registered provider
      await ExtrinsicHelper.createMsa(nonApprovedProvider).signAndSend();

      // Generate intermediary hashes
      const { a: intermediaryHashA, b: intermediaryHashB } = getIntermediaryHashes(
        recoverySecret,
        ContactType.EMAIL,
        testEmail
      );

      // Create add key payload
      const addKeyData: AddKeyData = {
        msaId: msaId,
        newPublicKey: getUnifiedAddress(newControlKeys),
      };
      const { payload: addKeyPayload, signature: newControlKeyProof } = await generateSignedAddKeyProofWithType(
        addKeyData,
        newControlKeys
      );

      // Attempt recovery with non-approved provider
      const recoverAccountOp = ExtrinsicHelper.recoverAccount(
        nonApprovedProvider,
        intermediaryHashA,
        intermediaryHashB,
        newControlKeyProof,
        addKeyPayload
      );

      await assert.rejects(recoverAccountOp.fundAndSend(fundingSource, false), {
        name: 'NotAuthorizedRecoveryProvider',
      });
    });

    it('should fail with invalid intermediary hashes', async function () {
      // Set up fresh user and recovery commitment for this test
      const { newControlKeys, msaId } = await setupUserWithRecoveryCommitment();

      // Use wrong recovery secret to generate invalid hashes
      const wrongRecoverySecret = generateRecoverySecret();
      const { a: wrongHashA, b: wrongHashB } = getIntermediaryHashes(wrongRecoverySecret, ContactType.EMAIL, testEmail);

      // Create add key payload
      const addKeyData: AddKeyData = {
        msaId: msaId,
        newPublicKey: getUnifiedAddress(newControlKeys),
      };
      const { payload: addKeyPayload, signature: newControlKeyProof } = await generateSignedAddKeyProofWithType(
        addKeyData,
        newControlKeys
      );

      // Attempt recovery with wrong hashes
      const recoverAccountOp = ExtrinsicHelper.recoverAccount(
        recoveryProviderKey,
        wrongHashA,
        wrongHashB,
        newControlKeyProof,
        addKeyPayload
      );

      await assert.rejects(recoverAccountOp.fundAndSend(fundingSource, false), {
        name: 'InvalidRecoveryCommitment',
      });
    });

    it('should fail with invalid new control key signature', async function () {
      // Set up fresh user and recovery commitment for this test
      const { msaId, recoverySecret } = await setupUserWithRecoveryCommitment();

      // Generate intermediary hashes
      const { a: intermediaryHashA, b: intermediaryHashB } = getIntermediaryHashes(
        recoverySecret,
        ContactType.EMAIL,
        testEmail
      );

      // Create add key payload but sign with wrong key
      const [wrongSigningKey, correctNewControlKeys] = await createAndFundKeypairs(
        fundingSource,
        ['wrongSigningKey', 'correctNewControlKeys'],
        1n * DOLLARS
      );

      const addKeyData: AddKeyData = {
        msaId: msaId,
        newPublicKey: getUnifiedAddress(correctNewControlKeys),
      };
      const { payload: addKeyPayload, signature: wrongNewControlKeyProof } = await generateSignedAddKeyProofWithType(
        addKeyData,
        wrongSigningKey
      );

      // Attempt recovery with wrong signature
      const recoverAccountOp = ExtrinsicHelper.recoverAccount(
        recoveryProviderKey,
        intermediaryHashA,
        intermediaryHashB,
        wrongNewControlKeyProof,
        addKeyPayload
      );

      await assert.rejects(recoverAccountOp.fundAndSend(fundingSource, false), {
        name: 'NewKeyOwnershipInvalidSignature',
      });
    });

    it('should fail when recovery commitment does not exist', async function () {
      // Create a new user without any recovery commitment
      const [userWithoutCommitment, newControlKeysForTest] = await createAndFundKeypairs(
        fundingSource,
        ['userWithoutCommitment', 'newControlKeysForTest'],
        2n * DOLLARS
      );

      const { target: noCommitmentMsaTarget } = await ExtrinsicHelper.createMsa(userWithoutCommitment).signAndSend();
      assert.notEqual(noCommitmentMsaTarget?.data.msaId, undefined, 'No commitment MSA Id not in expected event');
      const noCommitmentMsaId = noCommitmentMsaTarget!.data.msaId;

      // Try to recover account that has no recovery commitment
      const dummySecret = generateRecoverySecret();
      const { a: dummyHashA, b: dummyHashB } = getIntermediaryHashes(
        dummySecret,
        ContactType.EMAIL,
        'dummy@example.com'
      );

      const addKeyData: AddKeyData = {
        msaId: noCommitmentMsaId,
        newPublicKey: getUnifiedAddress(newControlKeysForTest),
      };
      const { payload: addKeyPayload, signature: newControlKeyProof } = await generateSignedAddKeyProofWithType(
        addKeyData,
        newControlKeysForTest
      );

      const recoverAccountOp = ExtrinsicHelper.recoverAccount(
        recoveryProviderKey,
        dummyHashA,
        dummyHashB,
        newControlKeyProof,
        addKeyPayload
      );

      await assert.rejects(recoverAccountOp.fundAndSend(fundingSource, false), {
        name: 'NoRecoveryCommitment',
      });
    });

    it('should fail when MSA ID in payload does not match recovery commitment owner', async function () {
      // Set up fresh user and recovery commitment for this test
      const { recoverySecret, newControlKeys: testNewControlKeys } = await setupUserWithRecoveryCommitment();

      // Create another user with their own recovery commitment
      const [anotherUser] = await createAndFundKeypairs(fundingSource, ['anotherUser'], 2n * DOLLARS);

      const { target: anotherMsaTarget } = await ExtrinsicHelper.createMsa(anotherUser).signAndSend();
      assert.notEqual(anotherMsaTarget?.data.msaId, undefined, 'Another MSA Id not in expected event');
      const anotherMsaId = anotherMsaTarget!.data.msaId;

      // Set up recovery commitment for the other user
      const anotherRecoverySecret = generateRecoverySecret();
      const anotherRecoveryCommitment = getRecoveryCommitment(anotherRecoverySecret, ContactType.EMAIL, testEmail);

      const anotherRecoveryCommitmentData: RecoveryCommitmentPayload = {
        discriminant: 'RecoveryCommitmentPayload',
        recoveryCommitment: anotherRecoveryCommitment,
      };
      const anotherPayload = await generateRecoveryCommitmentPayload({ ...anotherRecoveryCommitmentData });
      const anotherCodecForPayload = ExtrinsicHelper.api.registry.createType(
        'PalletMsaRecoveryCommitmentPayload',
        anotherPayload
      );
      const anotherOwnerSig = signPayloadSr25519(anotherUser, anotherCodecForPayload);

      const addAnotherRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(
        anotherUser,
        anotherOwnerSig,
        anotherPayload
      );
      await addAnotherRecoveryCommitmentOp.fundAndSend(fundingSource, false);

      // Generate intermediary hashes for original user's recovery secret (not the other user's)
      const { a: intermediaryHashA, b: intermediaryHashB } = getIntermediaryHashes(
        recoverySecret,
        ContactType.EMAIL,
        testEmail
      );

      // Try to recover but specify wrong MSA ID in the payload
      // This should fail because the intermediary hashes are for the first user's recovery commitment,
      // but we're trying to apply them to the second user's MSA ID
      const addKeyData: AddKeyData = {
        msaId: anotherMsaId, // Wrong MSA ID
        newPublicKey: getUnifiedAddress(testNewControlKeys),
      };
      const { payload: addKeyPayload, signature: newControlKeyProof } = await generateSignedAddKeyProofWithType(
        addKeyData,
        testNewControlKeys
      );

      const recoverAccountOp = ExtrinsicHelper.recoverAccount(
        recoveryProviderKey,
        intermediaryHashA,
        intermediaryHashB,
        newControlKeyProof,
        addKeyPayload
      );

      await assert.rejects(recoverAccountOp.fundAndSend(fundingSource, false), {
        name: 'InvalidRecoveryCommitment',
      });
    });
  });
});
