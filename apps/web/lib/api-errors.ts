export class ApiError extends Error {
  constructor(
    public message: string,
    public status: number = 400,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export function throwApiError(message: string, status: number = 400): never {
  throw new ApiError(message, status);
}
