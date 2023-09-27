import { unixfs } from '@helia/unixfs';
import { createHelia } from 'helia';

export async function ipfsCid(content: string, path: string) {
  // create a Helia node
  const helia = await createHelia({ start: false });

  // create a filesystem on top of Helia, in this case it's UnixFS
  const fs = unixfs(helia);

  // we will use this TextEncoder to turn strings into Uint8Arrays
  const encoder = new TextEncoder();

  // add the bytes to your node and receive a unique content identifier
  const cid = await fs.addFile({
    path,
    content: encoder.encode(content)
  });

  return cid;
}
