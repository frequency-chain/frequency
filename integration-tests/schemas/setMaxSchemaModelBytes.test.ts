import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import assert from "assert";
import { firstValueFrom } from "rxjs";
import { keyring } from "../scaffolding/apiConnection";
import { Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";

describe("#setMaxSchemaModelBytes", function () {
    let keys: KeyringPair;

    before(async function () {
        const sudoKey = (await firstValueFrom(ExtrinsicHelper.api.query.sudo.key())).unwrap();
        keys = keyring.getPair(sudoKey.toString());
    })

    it("should fail to set the schema size because of lack of root authority (BadOrigin)", async function () {
        const operation = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(1000000), keys);
        await assert.rejects(operation.signAndSend(), { name: 'BadOrigin' });
    });

    it("should fail to set max schema size > hard-coded limit (SchemaModelMaxBytesBoundedVecLimit)", async function () {
        const limit = ExtrinsicHelper.api.consts.schemas.schemaModelMaxBytesBoundedVecLimit.toBigInt();
        const op = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(limit + 1n), keys);
        await assert.rejects(op.sudoSignAndSend(), { name: 'ExceedsMaxSchemaModelBytes'});
    })

    it("should successfully set the max schema size", async function () {
        const size = (await firstValueFrom(ExtrinsicHelper.api.query.schemas.governanceSchemaModelMaxBytes())).toBigInt();
        const op = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(size + 1n), keys);
        await op.sudoSignAndSend();
        assert.equal(true, true, 'operation should not have thrown error');
        const newSize = (await firstValueFrom(ExtrinsicHelper.api.query.schemas.governanceSchemaModelMaxBytes())).toBigInt();
        assert.equal(size + 1n, newSize, 'new size should have been set');
    });
});
