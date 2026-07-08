export type IconButtonSize = "xs" | "sm" | "md" | "lg" | "custom";
export type IconButtonVariant = "default" | "danger" | "success";

const iconButtonSizes: Record<IconButtonSize, string> = {
  xs: "h-7 w-7",
  sm: "h-8 w-8",
  md: "h-9 w-9",
  lg: "h-10 w-10",
  custom: "",
};

const iconButtonVariants: Record<IconButtonVariant, string> = {
  default:
    "text-muted hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground",
  danger:
    "text-danger-badge-foreground hover:border-danger-badge-foreground hover:bg-danger-badge",
  success:
    "text-good-badge-foreground hover:border-good-badge-foreground hover:bg-good-badge",
};

export function iconButtonClassName({
  size = "sm",
  variant = "default",
  className,
}: {
  readonly size?: IconButtonSize;
  readonly variant?: IconButtonVariant;
  readonly className?: string;
} = {}): string {
  return [
    "inline-flex shrink-0 cursor-pointer items-center justify-center rounded-md border border-interactive-border bg-surface/70 shadow-sm shadow-black/[0.03] transition disabled:cursor-not-allowed disabled:border-disabled disabled:bg-transparent disabled:text-disabled-foreground disabled:opacity-50",
    iconButtonSizes[size],
    iconButtonVariants[variant],
    className,
  ]
    .filter(Boolean)
    .join(" ");
}

export function textButtonClassName(className?: string): string {
  return [
    "cursor-pointer rounded-md border border-interactive-border bg-surface/70 px-4 py-2 text-center text-sm font-medium text-foreground shadow-sm shadow-black/[0.03] transition hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground disabled:cursor-not-allowed disabled:border-disabled disabled:bg-transparent disabled:text-disabled-foreground disabled:opacity-50",
    className,
  ]
    .filter(Boolean)
    .join(" ");
}
