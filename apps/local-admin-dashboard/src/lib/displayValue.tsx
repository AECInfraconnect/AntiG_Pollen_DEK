import { isValidElement, type ReactNode } from "react";

export function formatDisplayValue(value: unknown): string {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "bigint") {
    return value.toString();
  }
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (Array.isArray(value)) {
    const text = value.map((item) => formatDisplayValue(item));
    return text.length ? text.join(", ") : "-";
  }
  if (typeof value === "object") {
    const record = value as Record<string, unknown>;
    const friendly =
      record.label ??
      record.name ??
      record.display_name ??
      record.title ??
      record.entity_id ??
      record.id ??
      record.type;
    if (friendly !== undefined && friendly !== null) {
      return formatDisplayValue(friendly);
    }
    try {
      return JSON.stringify(value);
    } catch {
      return String(value);
    }
  }
  return String(value);
}

export function renderDisplayValue(value: ReactNode): ReactNode {
  if (isValidElement(value)) return value;
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "bigint"
  ) {
    return value;
  }
  if (Array.isArray(value)) {
    const items = value.map((item) => formatDisplayValue(item));
    return items.map((item, index) => (
      <span key={`${item}-${index}`}>
        {item}
        {index < items.length - 1 ? ", " : ""}
      </span>
    ));
  }
  return formatDisplayValue(value);
}
