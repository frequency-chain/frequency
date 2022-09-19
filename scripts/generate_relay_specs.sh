<<<<<<< HEAD
#!/usr/bin/env bash

set -e

docker run parity/polkadot:v0.9.27 build-spec --disable-default-bootnode --chain rococo-local --raw > ./resources/rococo-local.json

||||||| parent of 3abc4b7 (Automation for generating rococo local spec file)
=======
#!/usr/bin/env bash

set -e

echo $(pwd)

docker run parity/polkadot:v0.9.27 build-spec --disable-default-bootnode --chain rococo-local --raw > ./resources/rococo-local.json
>>>>>>> 3abc4b7 (Automation for generating rococo local spec file)
