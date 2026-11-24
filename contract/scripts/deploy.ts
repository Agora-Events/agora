#!/usr/bin/env tsx
/**
 * Stellar Expert Testnet Contract Verification & Explorer Helper
 * Issue #1539
 *
 * Provides a CLI command and programmatic API to inspect, verify,
 * and view deployed Soroban smart contracts on the Stellar Expert Explorer:
 *   https://stellar.expert/explorer/testnet/contract/{id}
 *
 * Usage:
 *   # Print all deployed contract URLs found in environment / config:
 *   npx tsx contract/scripts/deploy.ts
 *
 *   # Pull up the Stellar Expert URL for a specific contract ID:
 *   npx tsx contract/scripts/deploy.ts CA3D522OI...
 *
 *   # View verification instructions:
 *   npx tsx contract/scripts/deploy.ts --verify CA3D522OI...
 *
 *   # Specify network (default: testnet):
 *   npx tsx contract/scripts/deploy.ts --network testnet <contract_id>
 */

import * as fs from "fs";
import * as path from "path";

export type StellarNetwork = "testnet" | "public" | "futurenet";

export const EXPLORER_BASE_URLS: Record<StellarNetwork, string> = {
  testnet: "https://stellar.expert/explorer/testnet",
  public: "https://stellar.expert/explorer/public",
  futurenet: "https://stellar.expert/explorer/futurenet",
};

/**
 * Returns the Stellar Expert explorer URL for a given contract ID.
 *
 * @param contractId - 56-character strkey starting with 'C'
 * @param network - Stellar network (default: "testnet")
 * @returns Explorer URL
 */
export function getStellarExpertContractUrl(
  contractId: string,
  network: StellarNetwork = "testnet",
): string {
  const base = EXPLORER_BASE_URLS[network] || EXPLORER_BASE_URLS.testnet;
  return `${base}/contract/${contractId.trim()}`;
}

/**
 * Returns the Stellar Expert explorer URL for a given account / wallet address.
 *
 * @param accountId - Public key starting with 'G'
 * @param network - Stellar network (default: "testnet")
 */
export function getStellarExpertAccountUrl(
  accountId: string,
  network: StellarNetwork = "testnet",
): string {
  const base = EXPLORER_BASE_URLS[network] || EXPLORER_BASE_URLS.testnet;
  return `${base}/account/${accountId.trim()}`;
}

/**
 * Returns the Stellar Expert explorer URL for a given transaction hash.
 *
 * @param txHash - Hex transaction hash
 * @param network - Stellar network (default: "testnet")
 */
export function getStellarExpertTxUrl(
  txHash: string,
  network: StellarNetwork = "testnet",
): string {
  const base = EXPLORER_BASE_URLS[network] || EXPLORER_BASE_URLS.testnet;
  return `${base}/tx/${txHash.trim()}`;
}

/**
 * Validates whether a string matches standard Soroban contract ID format (C... 56 chars).
 */
export function isValidContractId(id: string): boolean {
  return /^C[A-Z2-7]{55}$/.test(id.trim());
}

/**
 * Reads and parses key-value pairs from an env file if present.
 */
function parseEnvFile(filePath: string): Record<string, string> {
  const result: Record<string, string> = {};
  if (!fs.existsSync(filePath)) return result;

  try {
    const content = fs.readFileSync(filePath, "utf-8");
    for (const line of content.split("\n")) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const eqIdx = trimmed.indexOf("=");
      if (eqIdx !== -1) {
        const key = trimmed.slice(0, eqIdx).trim();
        const val = trimmed.slice(eqIdx + 1).trim().replace(/^["']|["']$/g, "");
        result[key] = val;
      }
    }
  } catch {
    // Ignore read errors
  }
  return result;
}

/**
 * Scans known configuration locations for deployed contract IDs.
 */
export function getDeployedContracts(): Record<string, string> {
  const contracts: Record<string, string> = {};

  // Check process.env first
  const envKeys = [
    "NEXT_PUBLIC_EVENT_CONTRACT_ID",
    "NEXT_PUBLIC_TICKET_CONTRACT_ID",
    "NEXT_PUBLIC_PRO_SUBSCRIPTION_CONTRACT_ID",
    "EVENT_REGISTRY_ID",
    "TICKET_PAYMENT_ID",
    "USDC_TOKEN_ADDRESS",
  ];
  for (const k of envKeys) {
    if (process.env[k] && isValidContractId(process.env[k]!)) {
      contracts[k] = process.env[k]!;
    }
  }

  // Check apps/web/.env.local
  const rootDir = path.resolve(__dirname, "../..");
  const webEnv = parseEnvFile(path.join(rootDir, "apps/web/.env.local"));
  for (const [k, v] of Object.entries(webEnv)) {
    if (isValidContractId(v) && !contracts[k]) {
      contracts[k] = v;
    }
  }

  // Check contract/.env.devnet
  const devnetEnv = parseEnvFile(path.join(rootDir, "contract/.env.devnet"));
  for (const [k, v] of Object.entries(devnetEnv)) {
    if (isValidContractId(v) && !contracts[k]) {
      contracts[k] = v;
    }
  }

  return contracts;
}

/**
 * Prints verification steps for a given contract.
 */
export function printVerificationSteps(contractId: string, network: StellarNetwork = "testnet") {
  const url = getStellarExpertContractUrl(contractId, network);
  console.log("\n==================================================================");
  console.log(` Stellar Expert Contract Verification (${network.toUpperCase()})`);
  console.log("==================================================================");
  console.log(`\nContract ID: ${contractId}`);
  console.log(`Explorer:    ${url}\n`);
  console.log("Steps to verify and interact with your contract:");
  console.log("  1. Open the explorer URL in your browser:");
  console.log(`     ${url}`);
  console.log("  2. In the 'Contract Info' tab, confirm the WASM Bytecode Hash matches");
  console.log("     the hash output during `cargo build --target wasm32-unknown-unknown --release`.");
  console.log("  3. View 'Contract Spec / Interface' to inspect exported entry points and data types.");
  console.log("  4. View 'Storage Entries' to verify instance and persistent state entries.");
  console.log("  5. In the 'Invocations' tab, test contract calls using a connected wallet (Freighter).");
  console.log("==================================================================\n");
}

// ── CLI Runner ─────────────────────────────────────────────────────────────────

export function runCli(args: string[] = process.argv.slice(2)) {
  let network: StellarNetwork = "testnet";
  let showVerify = false;
  let targetContractId: string | null = null;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--network" || arg === "-n") {
      network = (args[++i] as StellarNetwork) || "testnet";
    } else if (arg === "--verify" || arg === "-v") {
      showVerify = true;
    } else if (arg === "--help" || arg === "-h") {
      console.log(`
Usage:
  npx tsx contract/scripts/deploy.ts [options] [contract_id]

Options:
  --network, -n <testnet|public>  Target Stellar network (default: testnet)
  --verify, -v                    Show step-by-step verification instructions
  --help, -h                      Show this help message

Examples:
  npx tsx contract/scripts/deploy.ts
  npx tsx contract/scripts/deploy.ts C...56chars...
  npx tsx contract/scripts/deploy.ts --verify C...56chars...
`);
      return;
    } else if (!arg.startsWith("-")) {
      targetContractId = arg;
    }
  }

  if (targetContractId) {
    if (!isValidContractId(targetContractId)) {
      console.warn(`Warning: '${targetContractId}' does not match standard 56-char contract ID format (C...).`);
    }

    const url = getStellarExpertContractUrl(targetContractId, network);
    if (showVerify) {
      printVerificationSteps(targetContractId, network);
    } else {
      console.log(`\nContract ID:    ${targetContractId}`);
      console.log(`Stellar Expert: ${url}\n`);
    }
    return;
  }

  // Display all discovered contracts
  const contracts = getDeployedContracts();
  console.log("\n==================================================================");
  console.log(` Agora Soroban Contracts — Stellar Expert Explorer (${network})`);
  console.log("==================================================================");

  if (Object.keys(contracts).length === 0) {
    console.log("\nNo deployed contract IDs found in environment or .env files.");
    console.log("Deploy contracts with scripts/deploy-testnet.sh or pass a contract ID:");
    console.log("  npx tsx contract/scripts/deploy.ts <CONTRACT_ID>\n");
    return;
  }

  for (const [name, id] of Object.entries(contracts)) {
    const url = getStellarExpertContractUrl(id, network);
    console.log(`\n  ${name}:`);
    console.log(`    ID:  ${id}`);
    console.log(`    URL: ${url}`);
  }

  console.log("\n==================================================================");
  console.log("Tip: Run with --verify <CONTRACT_ID> for verification steps.\n");
}

if (require.main === module || (typeof process !== "undefined" && process.argv[1]?.endsWith("deploy.ts"))) {
  runCli();
}
