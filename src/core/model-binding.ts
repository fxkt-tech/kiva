export type ModelBindingSnapshot = {
  readonly provider: string;
  readonly model: string;
  readonly responseFormat: "json";
  readonly fallbackModel?: string;
};

export function validateModelBindingSnapshot(
  value: unknown,
  path: string,
): void {
  if (value === null) {
    return;
  }

  if (!isPlainObject(value)) {
    throw new Error(`${path} must be an object or null`);
  }
  const binding = value as Partial<ModelBindingSnapshot>;

  if ("temperature" in binding || "maxTokens" in binding) {
    throw new Error(`${path} must not configure temperature or maxTokens`);
  }

  assertExactObjectKeys(
    value,
    path,
    ["provider", "model", "responseFormat"],
    ["fallbackModel"],
  );

  if (!isStableString(binding.provider)) {
    throw new Error(`${path}.provider must be set`);
  }

  if (!isStableString(binding.model)) {
    throw new Error(`${path}.model must be set`);
  }

  if (binding.responseFormat !== "json") {
    throw new Error(`${path}.responseFormat must be json`);
  }

  if (
    binding.fallbackModel !== undefined &&
    !isStableString(binding.fallbackModel)
  ) {
    throw new Error(`${path}.fallbackModel must be a non-empty string`);
  }
}

export function isPlainObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function assertExactObjectKeys(
  value: Record<string, unknown>,
  path: string,
  required: readonly string[],
  optional: readonly string[] = [],
): void {
  const missing = required.filter((key) => !(key in value));
  const allowed = new Set([...required, ...optional]);
  const unknown = Object.keys(value).filter((key) => !allowed.has(key));
  if (missing.length > 0 || unknown.length > 0) {
    throw new Error(
      `${path} keys are invalid (missing: ${missing.join(", ") || "none"}; unknown: ${unknown.join(", ") || "none"})`,
    );
  }
}

function isStableString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0 && value === value.trim();
}
