import { Connection, PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider } from '@coral-xyz/anchor';
import { IDL } from './btb_token_sale.json'; // You'll need to generate this IDL file

const programID = new PublicKey('F4JnCD9KASp74g2zCg8GkoSj1boKKzCcZvq9Fjs4LzBz');

export function getProvider(connection: Connection, wallet: any) {
  const provider = new AnchorProvider(
    connection,
    wallet,
    AnchorProvider.defaultOptions()
  );
  return provider;
}

export function getProgram(connection: Connection, wallet: any) {
  const provider = getProvider(connection, wallet);
  const program = new Program(IDL, programID, provider);
  return program;
}

