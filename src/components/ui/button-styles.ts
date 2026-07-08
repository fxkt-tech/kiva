type IconButtonSize = "xs" | "sm" | "md" | "lg" | "custom";
type IconButtonVariant = "default" | "danger" | "success";

const iconButtonSizes: Record<IconButtonSize, string> = {
  xs: "h-7 w-7",
  sm: "h-8 w-8",
  md: "h-9 w-9",
  lg: "h-10 w-10",
  custom: "",
};

const iconButtonVariants: Record<IconButtonVariant, string> = {
  default:
    "text-muted hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground focus-visible:ring-accent/35",
  danger:
    "text-danger-badge-foreground hover:border-danger-badge-foreground hover:bg-danger-badge focus-visible:ring-danger-badge-foreground/35",
  success:
    "text-good-badge-foreground hover:border-good-badge-foreground hover:bg-good-badge focus-visible:ring-good-badge-foreground/35",
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
    "inline-flex shrink-0 items-center justify-center rounded-md border border-interactive-border bg-surface/70 shadow-sm shadow-black/[0.03] transition focus-visible:outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:border-disabled disabled:bg-transparent disabled:text-disabled-foreground disabled:opacity-50",
    iconButtonSizes[size],
    iconButtonVariants[variant],
    className,
  ]
    .filter(Boolean)
    .join(" ");
}

export function textButtonClassName(className?: string): string {
  return [
    "rounded-md border border-interactive-border bg-surface/70 px-4 py-2 text-center text-sm font-medium text-foreground shadow-sm shadow-black/[0.03] transition hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/35 disabled:cursor-not-allowed disabled:border-disabled disabled:bg-transparent disabled:text-disabled-foreground disabled:opacity-50",
    className,
  ]
    .filter(Boolean)
    .join(" ");
}
