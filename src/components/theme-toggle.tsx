"use client";

import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { useEffect, useState } from "react";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const dark = mounted && resolvedTheme === "dark";
  const nextTheme = dark ? "light" : "dark";

  return (
    <button
      aria-label={dark ? "Switch to light theme" : "Switch to dark theme"}
      className="inline-flex h-[38px] w-[38px] shrink-0 items-center justify-center rounded-md border border-interactive-border bg-surface/70 text-muted transition hover:border-interactive-border-hover hover:bg-surface-muted hover:text-foreground"
      onClick={() => setTheme(nextTheme)}
      title={dark ? "Light theme" : "Dark theme"}
      type="button"
    >
      <Sun
        aria-hidden="true"
        className={dark ? "h-4 w-4" : "hidden h-4 w-4"}
      />
      <Moon
        aria-hidden="true"
        className={dark ? "hidden h-4 w-4" : "h-4 w-4"}
      />
    </button>
  );
}
