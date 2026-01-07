// JavaScript implementation of ERC-55 ("Mixed-case checksum address encoding") using noble-hashes
// https://ercs.ethereum.org/ERCS/erc-55

// https://github.com/paulmillr/noble-hashes
import { keccak_256 } from '@noble/hashes/sha3';
// const { keccak_256 } = await import('https://esm.sh/@noble/hashes@1.4.0/esm/sha3.js');

function toChecksumAddress(address) {
  address = address.toLowerCase().replace('0x', '');
  // Hash the address (treat it as UTF-8) and return as a hex string
  const hash = [...keccak_256(address)].map(v => v.toString(16).padStart(2, '0')).join('');
  let ret = '0x';

  for (let i = 0; i < 40; i++) {
    if (parseInt(hash[i], 16) >= 8) {
      ret += address[i].toUpperCase();
    }
    else {
      ret += address[i];
    }
  }

  return ret;
}

let address = toChecksumAddress(process.argv[2]);
console.log(`Checksummed address: ${address}`);

