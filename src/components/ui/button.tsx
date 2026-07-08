import type { ButtonHTMLAttributes, ReactNode } from "react";
import {
  iconButtonClassName,
  type IconButtonSize,
  type IconButtonVariant,
  textButtonClassName,
} from "./button-styles";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  readonly children: ReactNode;
  readonly buttonStyle?: "text" | "icon";
  readonly iconSize?: IconButtonSize;
  readonly variant?: IconButtonVariant;
  readonly unstyled?: boolean;
};

export function Button({
  buttonStyle = "text",
  children,
  className,
  iconSize = "sm",
  type = "button",
  unstyled = false,
  variant = "default",
  ...props
}: ButtonProps) {
  const buttonClassName = unstyled
    ? className
    : buttonStyle === "icon"
      ? iconButtonClassName({ size: iconSize, variant, className })
      : textButtonClassName(className);

  return (
    <button className={buttonClassName} type={type} {...props}>
      {children}
    </button>
  );
}
