import {
  Contract,
  Keypair,
  Networks,
  TransactionBuilder,
  nativeToScVal,
  rpc,
} from "@stellar/stellar-sdk";

const STELLAR_CONTRACT_ADDRESS = process.env.STELLAR_CONTRACT_ADDRESS;
const STELLAR_SOURCE_SECRET = process.env.STELLAR_SOURCE_SECRET;
const STELLAR_RPC_URL = process.env.STELLAR_RPC_URL || "https://soroban-testnet.stellar.org";
const STELLAR_NETWORK_PASSPHRASE =
  process.env.STELLAR_NETWORK_PASSPHRASE || Networks.TESTNET;

function requireEnv(value: string | undefined, key: string): string {
  if (!value) {
    throw new Error(`Missing required environment variable: ${key}`);
  }
  return value;
}

export async function mintTicket(eventId: string, buyer: string, qty: number) {
  if (!eventId || !buyer || !Number.isInteger(qty) || qty <= 0) {
    throw new Error("Invalid mint ticket parameters");
  }

  const contractAddress = requireEnv(STELLAR_CONTRACT_ADDRESS, "STELLAR_CONTRACT_ADDRESS");
  const sourceSecret = requireEnv(STELLAR_SOURCE_SECRET, "STELLAR_SOURCE_SECRET");

  const sourceKeypair = Keypair.fromSecret(sourceSecret);
  const server = new rpc.Server(STELLAR_RPC_URL);
  const sourceAccount = await server.getAccount(sourceKeypair.publicKey());

  const contract = new Contract(contractAddress);
  const tx = new TransactionBuilder(sourceAccount, {
    fee: "100",
    networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        "mint_ticket",
        nativeToScVal(eventId, { type: "string" }),
        nativeToScVal(buyer, { type: "address" }),
        nativeToScVal(qty, { type: "u32" }),
      ),
    )
    .setTimeout(30)
    .build();

  tx.sign(sourceKeypair);
  const preparedTx = await server.prepareTransaction(tx);
  preparedTx.sign(sourceKeypair);
  const submitted = await server.sendTransaction(preparedTx);

  return {
    transactionXdr: preparedTx.toXDR(),
    ticketId: `ticket_${submitted.hash || Date.now().toString()}`,
  };
}
