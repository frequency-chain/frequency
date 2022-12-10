import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import { connect, createKeys } from "../scaffolding/apiConnection"
import { KeyringPair } from "@polkadot/keyring/types";
import { createSchema, addIPFSMessage, createMsa } from "../scaffolding/extrinsicHelpers";
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import assert from "assert";
import { GenericEvent } from "@polkadot/types";
import { EventMap } from "../scaffolding/helpers";
import { Event } from "@polkadot/types/interfaces";

describe("Add Messages", () => {
    let api: ApiRx;
    let keys: KeyringPair;

    before(async () => {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
    })

    after(() => {
        api.disconnect()
    })

    it("should successfully add an IPFS message", async () => {
        keys = createKeys("//Eve")

        // Create MSA
        const createMsaEvents = await createMsa(api, keys);
        assert.notEqual(createMsaEvents["msa.MsaCreated"], undefined);

        // Create a schema
        const createSchemaEvents = await createSchema(api, keys, PARQUET_BROADCAST, "Parquet", "IPFS");
        const schemaId = createSchemaEvents["schemas.SchemaCreated"][1];

        // This is how the IPFS data was created:
        //
        // echo "This is a test of Frequency." > frequency_test
        // % ipfs add frequency_test 
        // added QmYzm8KGxRHr7nGn5g5Z9Zv9r8nN5WNn7Ajya6x7RxmAB1 frequency_test
        // % wc -c frequency_test 
        // 29 frequency_test
  
        const payload = "This is a test of Frequency.";
        const chainEvents: EventMap = await addIPFSMessage(api, keys, schemaId, "QmYzm8KGxRHr7nGn5g5Z9Zv9r8nN5WNn7Ajya6x7RxmAB1", payload.length + 1);

        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);

    }).timeout(15000)
})

