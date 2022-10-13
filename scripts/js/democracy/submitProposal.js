const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api")

async function propose(proposalHash, deposit = 100_000_000_000_000) {
    const provider = new WsProvider("ws://0.0.0.0:9944");
    const api = await ApiPromise.create({ provider });
    const keyring = new Keyring({ type: "sr25519" });
    const sudo = keyring.addFromUri("//Alice");

    return api.tx.democracy.propose(proposalHash, deposit)
        .signAndSend(sudo, (result) => {
            console.log("Proposal made...")
            if (result.status.isFinalized) {
                console.log("Success!");
                console.log(result.events.map(e => e.get("event").get("data")));
            } else if (result.isError) {
                console.log(`Transaction Error`);
                process.exit()
            }
        })
}

module.exports = propose;
