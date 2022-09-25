import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { AnchorEscrow } from '../target/types/anchor_escrow';
import { PublicKey, SystemProgram, Transaction, Connection, Commitment } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { assert } from "chai";

describe('anchor-escrow', () => {
  const commitment: Commitment = 'processed';
  const connection = new Connection('https://api.devnet.solana.com', { commitment, wsEndpoint: 'wss://api.devnet.solana.com/' });
  const options = anchor.Provider.defaultOptions();
  const wallet = NodeWallet.local();
  const provider = new anchor.Provider(connection, wallet, options);

  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorEscrow as Program<AnchorEscrow>;

  let mintA = null as Token;
  let mintB = null as Token;
  let ITokenAccountA = null;
  let ITokenAccountB = null;
  let TTokenAccountA = null;
  let TTokenAccountB = null;
  let v_account_pda = null;
  let v_account_bump = null;
  let v_authority_pda = null;

  const takerAmount = 1000;
  const initializerAmount = 500;

  const escrowAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializerMainAccount = anchor.web3.Keypair.generate();
  const takerMainAccount = anchor.web3.Keypair.generate();

  it("Initialize program state", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 1000000000),
      "processed"
    );

    // Fund Main Accounts
    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: initializerMainAccount.publicKey,
            lamports: 100000000,
          }),
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: takerMainAccount.publicKey,
            lamports: 100000000,
          })
        );
        return tx;
      })(),
      [payer]
    );

    mintA = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    mintB = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    ITokenAccountA = await mintA.createAccount(initializerMainAccount.publicKey);
    TTokenAccountA = await mintA.createAccount(takerMainAccount.publicKey);

    ITokenAccountB = await mintB.createAccount(initializerMainAccount.publicKey);
    TTokenAccountB = await mintB.createAccount(takerMainAccount.publicKey);

    await mintA.mintTo(
      ITokenAccountA,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerAmount
    );

    await mintB.mintTo(
      TTokenAccountB,
      mintAuthority.publicKey,
      [mintAuthority],
      takerAmount
    );

    let _ITokenAccountA = await mintA.getAccountInfo(ITokenAccountA);
    let _TTokenAccountB = await mintB.getAccountInfo(TTokenAccountB);

    assert.ok(_ITokenAccountA.amount.toNumber() == initializerAmount);
    assert.ok(_TTokenAccountB.amount.toNumber() == takerAmount);
  });

  it("Initialize escrow", async () => {
    const [_v_account_pda, _v_account_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("token-seed"))],
      program.programId
    );
    v_account_pda = _v_account_pda;
    v_account_bump = _v_account_bump;

    const [_v_authority_pda, _vault_authority_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("escrow"))],
      program.programId
    );
    v_authority_pda = _v_authority_pda;

    await program.rpc.initialize(
      v_account_bump,
      new anchor.BN(initializerAmount),
      new anchor.BN(takerAmount),
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          vaultAccount: v_account_pda,
          mint: mintA.publicKey,
          initializerDepositTokenAccount: ITokenAccountA,
          initializerReceiveTokenAccount: ITokenAccountB,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(escrowAccount),
        ],
        signers: [escrowAccount, initializerMainAccount],
      }
    );

    let _vault = await mintA.getAccountInfo(v_account_pda);

    let _escrowAccount = await program.account.escrowAccount.fetch(
      escrowAccount.publicKey
    );

    // Check that the new owner is the PDA.
    assert.ok(_vault.owner.equals(v_authority_pda));

    // Check that the values in the escrow account match what we expect.
    assert.ok(_escrowAccount.initializerKey.equals(initializerMainAccount.publicKey));
    assert.ok(_escrowAccount.initializerAmount.toNumber() == initializerAmount);
    assert.ok(_escrowAccount.takerAmount.toNumber() == takerAmount);
    assert.ok(
      _escrowAccount.initializerDepositTokenAccount.equals(ITokenAccountA)
    );
    assert.ok(
      _escrowAccount.initializerReceiveTokenAccount.equals(ITokenAccountB)
    );
  });

  it("Exchange escrow state", async () => {
    await program.rpc.exchange({
      accounts: {
        taker: takerMainAccount.publicKey,
        takerDepositTokenAccount: TTokenAccountB,
        takerReceiveTokenAccount: TTokenAccountA,
        initializerDepositTokenAccount: ITokenAccountA,
        initializerReceiveTokenAccount: ITokenAccountB,
        initializer: initializerMainAccount.publicKey,
        escrowAccount: escrowAccount.publicKey,
        vaultAccount: v_account_pda,
        vaultAuthority: v_authority_pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [takerMainAccount]
    });

    let _TTokenAccountA = await mintA.getAccountInfo(TTokenAccountA);
    let _TTokenAccountB = await mintB.getAccountInfo(TTokenAccountB);
    let _ITokenAccountA = await mintA.getAccountInfo(ITokenAccountA);
    let _ITokenAccountB = await mintB.getAccountInfo(ITokenAccountB);

    assert.ok(_TTokenAccountA.amount.toNumber() == initializerAmount);
    assert.ok(_ITokenAccountA.amount.toNumber() == 0);
    assert.ok(_ITokenAccountB.amount.toNumber() == takerAmount);
    assert.ok(_TTokenAccountB.amount.toNumber() == 0);
  });

  it("Initialize escrow and cancel escrow", async () => {
    // Put back tokens into initializer token A account.
    await mintA.mintTo(
      ITokenAccountA,
      mintAuthority.publicKey,
      [mintAuthority],
      initializerAmount
    );

    await program.rpc.initialize(
      v_account_bump,
      new anchor.BN(initializerAmount),
      new anchor.BN(takerAmount),
      {
        accounts: {
          initializer: initializerMainAccount.publicKey,
          vaultAccount: v_account_pda,
          mint: mintA.publicKey,
          initializerDepositTokenAccount: ITokenAccountA,
          initializerReceiveTokenAccount: ITokenAccountB,
          escrowAccount: escrowAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(escrowAccount),
        ],
        signers: [escrowAccount, initializerMainAccount],
      }
    );

    // Cancel the escrow.
    await program.rpc.cancel({
      accounts: {
        initializer: initializerMainAccount.publicKey,
        initializerDepositTokenAccount: ITokenAccountA,
        vaultAccount: v_account_pda,
        vaultAuthority: v_authority_pda,
        escrowAccount: escrowAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [initializerMainAccount]
    });

    // Check the final owner should be the provider public key.
    const _ITokenAccountA = await mintA.getAccountInfo(ITokenAccountA);
    assert.ok(_ITokenAccountA.owner.equals(initializerMainAccount.publicKey));

    // Check all the funds are still there.
    assert.ok(_ITokenAccountA.amount.toNumber() == initializerAmount);
  });
});
