import { toast } from "sonner";

// ─── Contract Error Code Mappings ───────────────────────────────────────────

/**
 * Maps Soroban contract error codes to human-readable messages.
 * Error codes are contract-specific integers defined in the contract source.
 *
 * event_registry contract errors:
 * - 101: SoldOut - Ticket tier has sold all available tickets
 * - 102: SaleNotActive - Event registration period is not open
 * - 103: InvalidTimestamp - Event date is invalid
 * - 104: AlreadyRegistered - User is already registered for this event
 * - 105: InvalidTicketTier - Requested ticket tier does not exist
 * - 107: Unauthorized - Caller is not authorized to perform this operation
 * - 108: ExceedsMaxSupply - Requested quantity exceeds maximum available supply
 * - 109: MaxPerUserExceeded - Requested quantity exceeds limit per user
 *
 * ticket_payment contract errors:
 * - 101: SoldOut - This ticket tier is currently sold out
 * - 102: SaleNotActive - Event registration period has closed
 * - 103: InsufficientFunds - Insufficient USDC balance to complete purchase
 * - 104: PaymentFailed - Transaction failed during payment processing
 * - 105: InvalidTicketTier - Requested ticket tier does not exist
 * - 106: TransferError - Failed to transfer funds from buyer
 */
export const contractErrorMessages: Record<number, string> = {
  101: "This ticket tier is currently sold out.",
  102: "Event registration period has closed.",
  103: "Event date or timestamp is invalid.",
  104: "You are already registered for this event.",
  105: "The requested ticket tier does not exist.",
  106: "Failed to transfer funds. Please check your balance and try again.",
  107: "You are not authorized to perform this action.",
  108: "Requested ticket quantity exceeds maximum available supply.",
  109: "You have reached the maximum allowed tickets per user for this tier.",
};

/**
 * Common Soroban and Stellar RPC error pattern mappings.
 */
export const commonRpcErrors: Array<{ pattern: RegExp | string; message: string }> = [
  {
    pattern: /tx_insufficient_fee|insufficient.*fee|fee.*insufficient|op_underfunded|tx_insufficient_balance/i,
    message: "Insufficient testnet XLM for fee. Please fund your wallet with testnet XLM to pay network transaction fees.",
  },
  {
    pattern: /tx_bad_auth|bad_auth|unauthorized/i,
    message: "Transaction authorization failed. Please ensure you are connected with the correct wallet account.",
  },
  {
    pattern: /tx_bad_seq|bad_seq/i,
    message: "Transaction sequence number mismatch. Please refresh and try again.",
  },
  {
    pattern: /tx_too_early|tx_too_late/i,
    message: "Transaction submission timed out. Please try again.",
  },
  {
    pattern: /budget.*exceeded|cpu_insufficient|memory_insufficient/i,
    message: "Transaction exceeded Soroban resource or computation budget.",
  },
  {
    pattern: /entry.*expired|state.*archived|storage.*expired/i,
    message: "Contract storage entry has expired. Please restore state before retrying.",
  },
  {
    pattern: /op_no_trust|no_trustline|trustline.*missing/i,
    message: "USDC trustline missing. Please establish a USDC trustline before completing payment.",
  },
  {
    pattern: /hostfunctionerror/i,
    message: "A smart contract invocation error occurred. Please verify your inputs and account balance.",
  },
  {
    pattern: /simulation error/i,
    message: "Transaction simulation failed. The smart contract rejected the transaction preconditions.",
  },
  {
    pattern: /tx_failed/i,
    message: "Your transaction was rejected by the Stellar network. Please check your account and try again.",
  },
];

/**
 * Friendly error message to display when contract error code is not recognized.
 */
export const DEFAULT_UNKNOWN_ERROR =
  "An error occurred during the transaction. Please try again or contact support.";

// ─── Error Extraction & Parsing ──────────────────────────────────────────────

export interface ParsedError {
  /** The human-readable error message to display to the user. */
  userMessage: string;
  /** The raw error code (e.g., 101) extracted from the contract error. */
  errorCode?: number;
  /** Full technical error details for logging. */
  technicalDetails: string;
  /** Whether this is a contract-specific error vs. generic network error. */
  isContractError: boolean;
  /** The original raw error for developer debugging. */
  rawError?: unknown;
}

/**
 * Checks if the runtime environment is development.
 */
function isDevMode(): boolean {
  return (
    typeof process !== "undefined" &&
    process.env &&
    process.env.NODE_ENV === "development"
  );
}

/**
 * Logs full technical error details to the console in development mode or for debugging.
 *
 * @param error - The original raw error object/string
 * @param context - Optional context string (e.g., "ticket purchase", "event registration")
 */
export function logTechnicalError(
  error: unknown,
  context: string = "Soroban operation",
): void {
  const timestamp = new Date().toISOString();
  
  // Always log raw RPC error in dev mode or whenever debugging
  if (isDevMode() || typeof window !== "undefined") {
    console.error(`[${timestamp}] [Soroban RPC Error] (${context}):`, error);
    if (error && typeof error === "object" && "stack" in error) {
      console.error("Stack trace:", (error as Error).stack);
    }
  }
}

/**
 * Parses a Soroban error object or string and extracts:
 * 1. Contract error code (if present)
 * 2. Human-readable error message
 * 3. Technical details for logging
 *
 * @param error - The Soroban error to parse
 * @returns ParsedError with user message and technical details
 */
export function parseSorobanError(error: unknown): ParsedError {
  const technicalDetails =
    typeof error === "string"
      ? error
      : error instanceof Error
      ? `${error.name}: ${error.message}`
      : JSON.stringify(error, null, 2);

  // ─── Try to extract contract error from error object ──────────────────────────

  if (error && typeof error === "object") {
    const errorObj = error as Record<string, unknown>;

    // Pattern 1: error.result?.resultMetaXdr contains error code
    if (errorObj.result && typeof errorObj.result === "object") {
      const result = errorObj.result as Record<string, unknown>;
      const errorCode = extractErrorCodeFromResult(result);
      if (errorCode !== null) {
        return {
          userMessage: contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
          errorCode,
          technicalDetails,
          isContractError: true,
          rawError: error,
        };
      }
    }

    // Pattern 2: error.message is a string containing error code pattern
    if (typeof errorObj.message === "string") {
      const errorCode = extractErrorCodeFromString(errorObj.message);
      if (errorCode !== null) {
        return {
          userMessage: contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
          errorCode,
          technicalDetails,
          isContractError: true,
          rawError: error,
        };
      }

      // Check common RPC errors in errorObj.message
      for (const entry of commonRpcErrors) {
        if (
          typeof entry.pattern === "string"
            ? errorObj.message.includes(entry.pattern)
            : entry.pattern.test(errorObj.message)
        ) {
          return {
            userMessage: entry.message,
            technicalDetails,
            isContractError: false,
            rawError: error,
          };
        }
      }
    }

    // Pattern 3: Direct error.error_code field
    if (typeof errorObj.error_code === "number") {
      const errorCode = errorObj.error_code;
      return {
        userMessage: contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
        errorCode,
        technicalDetails,
        isContractError: true,
        rawError: error,
      };
    }

    // Pattern 4: Transaction failure status
    if (errorObj.status === "FAILED" || errorObj.status === "ERROR") {
      const errorCode = extractErrorCodeFromTransactionStatus(errorObj);
      if (errorCode !== null) {
        return {
          userMessage: contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
          errorCode,
          technicalDetails,
          isContractError: true,
          rawError: error,
        };
      }
    }
  }

  // ─── Try to extract from string or match common RPC errors ───────────────────

  const errorString = String(error);

  const errorCode = extractErrorCodeFromString(errorString);
  if (errorCode !== null) {
    return {
      userMessage: contractErrorMessages[errorCode] || DEFAULT_UNKNOWN_ERROR,
      errorCode,
      technicalDetails,
      isContractError: true,
      rawError: error,
    };
  }

  for (const entry of commonRpcErrors) {
    if (
      typeof entry.pattern === "string"
        ? errorString.includes(entry.pattern)
        : entry.pattern.test(errorString)
    ) {
      return {
        userMessage: entry.message,
        technicalDetails,
        isContractError: false,
        rawError: error,
      };
    }
  }

  // ─── Fallback for unrecognized errors ───────────────────────────────────────

  return {
    userMessage: DEFAULT_UNKNOWN_ERROR,
    technicalDetails,
    isContractError: false,
    rawError: error,
  };
}

// ─── Helper Functions for Error Code Extraction ──────────────────────────────

function extractErrorCodeFromResult(result: Record<string, unknown>): number | null {
  if (typeof result.code === "number") {
    return result.code;
  }

  if (typeof result.resultMetaXdr === "string") {
    const match = result.resultMetaXdr.match(/Error\(Contract,\s*#?(\d+)\)/);
    if (match) {
      return parseInt(match[1], 10);
    }
  }

  return null;
}

function extractErrorCodeFromString(str: string): number | null {
  let match = str.match(/Error\(Contract,\s*#?(\d+)\)/i);
  if (match) {
    return parseInt(match[1], 10);
  }

  match = str.match(/(?:Contract\s+)?error(?:\s+code)?:\s*#?(\d+)/i);
  if (match) {
    return parseInt(match[1], 10);
  }

  match = str.match(/#(\d{3})$/);
  if (match) {
    const code = parseInt(match[1], 10);
    if (code >= 100 && code <= 999) {
      return code;
    }
  }

  return null;
}

function extractErrorCodeFromTransactionStatus(
  status: Record<string, unknown>,
): number | null {
  if (typeof status.error_code === "number") {
    return status.error_code;
  }

  if (typeof status.resultMetaXdr === "string") {
    return extractErrorCodeFromString(status.resultMetaXdr);
  }

  if (typeof status.resultXdr === "string") {
    return extractErrorCodeFromString(status.resultXdr);
  }

  if (typeof status.error === "string") {
    return extractErrorCodeFromString(status.error);
  }

  return null;
}

// ─── Combined Parsing, Logging & Toast Display ───────────────────────────────

/**
 * Parses a Soroban error and logs technical details while returning the
 * friendly user message.
 */
export function parseAndLogError(
  error: unknown,
  context: string = "transaction",
): ParsedError {
  const parsed = parseSorobanError(error);
  logTechnicalError(error, context);
  return parsed;
}

/**
 * Handles transaction failure by parsing the Soroban error, logging the raw error
 * to console for debugging, and displaying a clear user-friendly toast message.
 *
 * @param error - The raw error from transaction execution or RPC
 * @param context - Context description (e.g. "ticket purchase")
 * @returns ParsedError
 */
export function showTransactionErrorToast(
  error: unknown,
  context: string = "transaction",
): ParsedError {
  const parsed = parseAndLogError(error, context);
  toast.error(parsed.userMessage);
  return parsed;
}
