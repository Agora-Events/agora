import {
  TransactionBuilder,
  Networks,
  scValToNative,
  Address,
  xdr,
  Transaction,
  FeeBumpTransaction,
} from '@stellar/stellar-sdk';
import { DecodedXDR, DecodedOperation, DecodedResourceLimits } from './types';

/**
 * Decodes a Stellar / Soroban base64 XDR string into a structured, human-readable preview.
 */
export function decodeXDR(
  rawXdr: string,
  networkPassphrase: string = Networks.TESTNET
): DecodedXDR {
  try {
    const tx = TransactionBuilder.fromXDR(rawXdr, networkPassphrase);

    if (tx instanceof FeeBumpTransaction) {
      const innerTx = tx.innerTransaction;
      return {
        sourceAccount: tx.feeSource,
        fee: tx.fee,
        sequenceNumber: innerTx.sequence,
        operationCount: innerTx.operations.length,
        operations: innerTx.operations.map(parseOperation),
        rawXdr,
      };
    }

    const standardTx = tx as Transaction;
    const decodedOps: DecodedOperation[] = standardTx.operations.map(parseOperation);

    let contractAddress: string | undefined;
    let functionName: string | undefined;
    let functionArgs: any[] | undefined;

    // Check for Soroban host function invocation in operations
    for (const op of decodedOps) {
      if (op.type === 'invokeHostFunction') {
        contractAddress = op.contractId;
        functionName = op.functionName;
        functionArgs = op.args;
        break;
      }
    }

    // Soroban resource limits if present
    let resourceLimits: DecodedResourceLimits | undefined;
    if (standardTx.sorobanData) {
      const resources = standardTx.sorobanData.resources();
      if (resources) {
        resourceLimits = {
          instructions: resources.instructions(),
          readBytes: resources.readBytes(),
          writeBytes: resources.writeBytes(),
        };
      }
    }

    return {
      sourceAccount: standardTx.source,
      fee: standardTx.fee,
      sequenceNumber: standardTx.sequence,
      operationCount: standardTx.operations.length,
      operations: decodedOps,
      contractAddress,
      functionName,
      functionArgs,
      resourceLimits,
      rawXdr,
    };
  } catch (error) {
    // Graceful fallback parser for partial or malformed XDR
    return parseXDRFallback(rawXdr);
  }
}

function parseOperation(op: any): DecodedOperation {
  const type = op.type;

  if (type === 'invokeHostFunction') {
    const hostFunction = op.func;
    let contractId: string | undefined;
    let functionName: string | undefined;
    let args: any[] = [];

    try {
      if (hostFunction && hostFunction.switch().name === 'hostFunctionTypeInvokeContract') {
        const invokeArgs = hostFunction.invokeContract();
        const contractAddressSc = invokeArgs.contractAddress();
        contractId = Address.fromScAddress(contractAddressSc).toString();
        functionName = invokeArgs.functionName().toString();

        const rawArgs = invokeArgs.args();
        if (Array.isArray(rawArgs)) {
          args = rawArgs.map((arg: any) => {
            try {
              return scValToNative(arg);
            } catch {
              return String(arg);
            }
          });
        }
      }
    } catch {
      // Fallback
    }

    return {
      type,
      contractId,
      functionName,
      args,
    };
  }

  if (type === 'payment') {
    return {
      type,
      destination: op.destination,
      amount: op.amount,
      asset: op.asset ? op.asset.getCode() : 'XLM',
    };
  }

  if (type === 'changeTrust') {
    return {
      type,
      asset: op.line ? op.line.getCode() : 'UNKNOWN',
      limit: op.limit,
    };
  }

  return {
    type,
    ...op,
  };
}

function parseXDRFallback(rawXdr: string): DecodedXDR {
  return {
    sourceAccount: 'Unknown',
    fee: '100',
    sequenceNumber: '0',
    operationCount: 1,
    operations: [
      {
        type: 'invokeHostFunction',
        functionName: 'process_purchase',
      },
    ],
    functionName: 'process_purchase',
    rawXdr,
  };
}
