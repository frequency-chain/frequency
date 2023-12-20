"""
This script is used to upgrade accounts to the new balance storage scheme
introduced in https://github.com/paritytech/substrate/pull/12951

Install the dependency https://github.com/polkascan/py-substrate-interface like:
 pip install substrate-interface
Then run it:
 python3 upgrade-accounts.py
"""

import json
import os
from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.exceptions import SubstrateRequestException

chain = SubstrateInterface(
    url="wss://rpc.rococo.frequency.xyz",
    # Using the public endpoint can get you rate-limited.
    # url="wss://kusama-rpc.polkadot.io",
    # These Parity internals are not limited.
    # url="wss://polkadot-try-runtime-node.parity-chains.parity.io:443"
)

print(f"Connected to {chain.name}: {chain.chain} v{chain.version}")

sender_uri = os.getenv('SENDER_URI', '//Alice')
sender = Keypair.create_from_uri(sender_uri)
print(f"Using sender account {sender.ss58_address}")


def main():
    """
    […] run though all accounts with reserved/locked funds on the system and call a
    particular transaction on them
    """
    accounts = []
    account_query = chain.query_map('System', 'Account', page_size=1000)

    NEW_LOGIC_FLAG = 0x80000000_00000000_00000000_00000000

    for (i, (id, info)) in enumerate(account_query):
        account = info['data']
        flags = account['flags'].decode()

        if flags & NEW_LOGIC_FLAG == 0:
            accounts.append(id.value)

        if i % 5000 == 0 and i > 0:
            percent = round((100 * len(accounts)) / (i + 1), 2)
            print(
                f"Checked {i} accounts; {len(accounts)} ({percent} %) are eligible for upgrade")

    print(f"Found {len(accounts)} eligible accounts in total")

    out_file = f"upgradable-accs-{chain.chain}.json"
    with open(out_file, 'w') as f:
        json.dump(accounts, f)
        print(f"Wrote accounts to '{out_file}'")

    # How many accounts each call should upgrade.
    accs_per_call = 1024
    weight_second = 1e12
    decimals = chain.token_decimals or 0

    for (i, chunk) in enumerate(chunks(accounts, accs_per_call)):
        call = chain.compose_call(
            call_module='Balances',
            call_function='upgrade_accounts',
            call_params={
                'who': chunk,
            }
        )
        extrinsic = chain.create_signed_extrinsic(call=call, keypair=sender)
        print(f"Extrinsic {i + 1}: upgrading {len(chunk)} accounts")

        try:
            receipt = chain.submit_extrinsic(
                extrinsic, wait_for_inclusion=True)
            print(f"Extrinsic included in block {receipt.block_hash}: "
                  f"consumed {receipt.weight['ref_time'] / weight_second} seconds of weight and "
                  f"paid {(receipt.total_fee_amount or 0) / 10**decimals} {chain.token_symbol}")
            if len(receipt.triggered_events) < len(chunk):
                print(
                    f"!! Emitted fewer events than expected: {len(receipt.triggered_events)} < {len(chunk)}")
        except SubstrateRequestException as e:
            print(f"Failed to submit extrinsic: {e}")
            raise e


def chunks(list, n):
    """
    Lazily split 'list' into 'n'-sized chunks.
    """
    for i in range(0, len(list), n):
        yield list[i:i + n]


if __name__ == "__main__":
    main()
