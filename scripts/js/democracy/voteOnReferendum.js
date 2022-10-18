const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api")

async function aye(referendumIndex, balance = 100) {
    return vote(true, referendumIndex, balance);
}

async function nay(referendumIndex, balance = 100) {
    return vote(false, referendumIndex, balance);
}

async function vote(aye, referendumIndex, balance) {
    const provider = new WsProvider("ws://0.0.0.0:9944");
    const api = await ApiPromise.create({ provider });
    const keyring = new Keyring({ type: "sr25519" });
    const sudo = keyring.addFromUri("//Alice");

    return api.tx.democracy.vote(referendumIndex, {
        Standard: {
            vote: {
                conviction: "None",
                aye
            },
            balance
        }}).signAndSend(sudo, (result) => {
            console.log(`Voting ${ aye ? "Aye" : "Nay" }...`);
            if (result.status.isFinalized) {
                console.log("Success!");
                console.log(result);
            } else if (result.isError) {
                console.log(`Transaction Error`);
                process.exit()
            }
        })
}

module.exports = {
    aye, nay
};
