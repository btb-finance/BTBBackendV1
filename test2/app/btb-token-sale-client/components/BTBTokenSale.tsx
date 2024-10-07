import { useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { getProgram } from '../utils/anchor-client';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

export const BTBTokenSale = () => {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [amount, setAmount] = useState('');
  const [isVested, setIsVested] = useState(false);

  const initializeSale = async () => {
    if (!wallet.publicKey) return;
    const program = getProgram(connection, wallet);
    const startTime = Math.floor(Date.now() / 1000);
    const endTime = startTime + 7 * 24 * 60 * 60; // 1 week from now
    const totalTokensForSale = new BN(800_000_000 * 1e6);

    try {
      const tx = await program.methods.initializeSale(
        new BN(startTime),
        new BN(endTime),
        totalTokensForSale
      )
      .accounts({
        sale: /* PDA for sale account */,
        owner: wallet.publicKey,
        btbMint: /* BTB token mint address */,
        usdtMint: /* USDT token mint address */,
        ownerBtbAccount: /* Owner's BTB token account */,
        saleVault: /* Sale vault account */,
        systemProgram: PublicKey.default,
        tokenProgram: PublicKey.default,
      })
      .rpc();
      console.log("Sale initialized:", tx);
    } catch (error) {
      console.error("Error initializing sale:", error);
    }
  };

  const processPurchase = async () => {
    if (!wallet.publicKey) return;
    const program = getProgram(connection, wallet);
    const amountBN = new BN(parseFloat(amount) * 1e6);

    try {
      const tx = await program.methods.processPurchase(amountBN, isVested)
      .accounts({
        sale: /* PDA for sale account */,
        buyer: wallet.publicKey,
        buyerUsdtAccount: /* Buyer's USDT token account */,
        buyerBtbAccount: /* Buyer's BTB token account */,
        saleUsdtAccount: /* Sale USDT account */,
        saleVault: /* Sale vault account */,
        vestingInfo: /* PDA for vesting info account */,
        systemProgram: PublicKey.default,
        tokenProgram: PublicKey.default,
      })
      .rpc();
      console.log("Purchase processed:", tx);
    } catch (error) {
      console.error("Error processing purchase:", error);
    }
  };

  const claimVestedTokens = async () => {
    if (!wallet.publicKey) return;
    const program = getProgram(connection, wallet);

    try {
      const tx = await program.methods.claimVestedTokens()
      .accounts({
        sale: /* PDA for sale account */,
        vestingInfo: /* PDA for vesting info account */,
        buyer: wallet.publicKey,
        buyerBtbAccount: /* Buyer's BTB token account */,
        saleVault: /* Sale vault account */,
        tokenProgram: PublicKey.default,
      })
      .rpc();
      console.log("Vested tokens claimed:", tx);
    } catch (error) {
      console.error("Error claiming vested tokens:", error);
    }
  };

  const emergencyWithdraw = async () => {
    if (!wallet.publicKey) return;
    const program = getProgram(connection, wallet);

    try {
      const tx = await program.methods.emergencyWithdraw()
      .accounts({
        sale: /* PDA for sale account */,
        owner: wallet.publicKey,
        ownerBtbAccount: /* Owner's BTB token account */,
        saleVault: /* Sale vault account */,
        tokenProgram: PublicKey.default,
      })
      .rpc();
      console.log("Emergency withdrawal completed:", tx);
    } catch (error) {
      console.error("Error performing emergency withdrawal:", error);
    }
  };

  return (
    <div>
      <h1>BTB Token Sale</h1>
      <WalletMultiButton />
      {wallet.connected && (
        <>
          <button onClick={initializeSale}>Initialize Sale</button>
          <div>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="Amount to purchase"
            />
            <label>
              <input
                type="checkbox"
                checked={isVested}
                onChange={(e) => setIsVested(e.target.checked)}
              />
              Vested Purchase
            </label>
            <button onClick={processPurchase}>Purchase Tokens</button>
          </div>
          <button onClick={claimVestedTokens}>Claim Vested Tokens</button>
          <button onClick={emergencyWithdraw}>Emergency Withdraw</button>
        </>
      )}
    </div>
  );
};

