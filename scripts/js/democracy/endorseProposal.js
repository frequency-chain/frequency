const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api")

async function endorse(proposalIndex) {
    const provider = new WsProvider("ws://0.0.0.0:9944");
    const api = await ApiPromise.create({ provider });
    const keyring = new Keyring({ type: "sr25519" });
    const sudo = keyring.addFromUri("//Alice");

    await api.tx.democracy.second(proposalIndex, 1)
        .signAndSend(sudo, (result) => {
            console.log("Endorsement made...")
            if (result.status.isFinalized) {
                console.log("Success!");
                console.log(result);
            } else if (result.isError) {
                console.log(`Transaction Error`);
                process.exit()
            }
        })
}

module.exports = endorse;
