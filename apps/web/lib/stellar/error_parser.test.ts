import {
  parseSorobanError,
  parseAndLogError,
  contractErrorMessages,
  DEFAULT_UNKNOWN_ERROR,
} from "./error_parser";

describe("Soroban error_parser", () => {
  it("translates insufficient fee errors to friendly message", () => {
    const error = "tx_insufficient_fee: fee below minimum required";
    const parsed = parseSorobanError(error);
    expect(parsed.userMessage).toContain("Insufficient testnet XLM for fee");
    expect(parsed.isContractError).toBe(false);
  });

  it("translates bad auth errors", () => {
    const error = "tx_bad_auth: signature failed";
    const parsed = parseSorobanError(error);
    expect(parsed.userMessage).toContain("Transaction authorization failed");
  });

  it("translates contract error code from string", () => {
    const error = "Error(Contract, #101)";
    const parsed = parseSorobanError(error);
    expect(parsed.errorCode).toBe(101);
    expect(parsed.userMessage).toBe(contractErrorMessages[101]);
    expect(parsed.isContractError).toBe(true);
  });

  it("translates contract error code from object message", () => {
    const error = { message: "Contract error: 102" };
    const parsed = parseSorobanError(error);
    expect(parsed.errorCode).toBe(102);
    expect(parsed.userMessage).toBe(contractErrorMessages[102]);
  });

  it("handles unknown errors gracefully", () => {
    const error = new Error("Something weird happened");
    const parsed = parseSorobanError(error);
    expect(parsed.userMessage).toBe(DEFAULT_UNKNOWN_ERROR);
    expect(parsed.isContractError).toBe(false);
  });

  it("parseAndLogError returns parsed error and logs to console", () => {
    const spy = jest.spyOn(console, "error").mockImplementation(() => {});
    const parsed = parseAndLogError("tx_insufficient_fee", "unit test");
    expect(parsed.userMessage).toContain("Insufficient testnet XLM for fee");
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });
});
