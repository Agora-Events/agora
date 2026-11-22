import { describe, it, expect } from "vitest";
import { StrKey } from "@stellar/stellar-sdk";
import { validateContractId } from "./config";

describe("validateContractId", () => {
  it("returns true for a valid contract ID", () => {
    const validContractId = StrKey.encodeContract(Buffer.alloc(32, 1));
    expect(validContractId.startsWith("C")).toBe(true);
    expect(validateContractId(validContractId)).toBe(true);
  });

  it("returns false for an invalid contract ID string", () => {
    expect(validateContractId("invalid-contract-id")).toBe(false);
    expect(validateContractId("")).toBe(false);
    expect(validateContractId("GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5")).toBe(false);
    expect(validateContractId("C12345")).toBe(false);
  });
});
