import assert from "assert";
import sinon from "sinon";

import { AVRO_GRAPH_CHANGE } from "../fixtures/schemaTypes";
import { AVRO, PARQUET } from "../fixtures/modelTypes";
import { ON_CHAIN, IPFS } from "../fixtures/payloadLocation";

import ApiWrapper from "./apiWrapper";

describe("#ApiWrapper", () => {
    let mockPolkadot;
    let mockKeys;
    let api;

    beforeEach(() => {
        api = new ApiWrapper(mockPolkadot, mockKeys);
    })

    describe("#createSchema", () => {
        beforeEach(() => {
            let mockTx = sinon.stub();
            mockPolkadot = {tx: { schemas: { createSchema: sinon.stub().resolves(mockTx) }}}
            api = new ApiWrapper(mockPolkadot, mockKeys);
            api._sendTx = sinon.stub().resolves();
        });

        it("should call the createSchema extrinsic", async () => {
            await api.createSchema(AVRO_GRAPH_CHANGE, AVRO, ON_CHAIN);
            assert.equal(api._sendTx.called, true);
        });
    });

    describe("#getEvent", () => {
        it("should fetch an event", () => {
            api._eventData["foo"] = "bar";
            assert.equal(api.getEvent("foo"), "bar");
        });

        it("should return undefined for an event that does not exist", () => {
            api._eventData["foo"] = "bar";
            assert.equal(api.getEvent("baz"), undefined);
        });
    });

    describe("#_handleTxResponse", () => {

        beforeEach(() => {
            sinon.spy(api, "_storeEvent");
        })
        it("should resolve for an inBlock event", async () => {
            await api._handleTxResponse({isInBlock: true, isFinalized: false}, [{event: {
                section: "foo",
                method: "bar",
                data: {value: 1}
            }}]);

            assert.equal(api._storeEvent.called, true);
        });
        it("should resolve for a Finalized event", () => {
            it("should resolve for an inBlock event", async () => {
                await api._handleTxResponse({isInBlock: false, isFinalized: true}, [{event: {
                    section: "foo",
                    method: "bar",
                    data: {value: 1}
                }}]);
    
                assert.equal(api._storeEvent.called, true);
            }); 
        });
    });

    describe("#_storeEvent", () => {
        it("should store events by section and method", () => {
            let event = {
                section: "foo",
                method: "bar",
                data: {value: 1}
            }
            api._storeEvent(event);
            assert.equal(api._eventData["foo.bar"], event);
        })
    });
})