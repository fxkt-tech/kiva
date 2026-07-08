"use client";

import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const dark = mounted && resolvedTheme === "dark";
  const nextTheme = dark ? "light" : "dark";

  return (
    <Button
      aria-label={dark ? "Switch to light theme" : "Switch to dark theme"}
      buttonStyle="icon"
      className="h-[38px] w-[38px]"
      iconSize="custom"
      onClick={() => setTheme(nextTheme)}
      title={dark ? "Light theme" : "Dark theme"}
    >
      <Sun
        aria-hidden="true"
        className={dark ? "h-4 w-4" : "hidden h-4 w-4"}
      />
      <Moon
        aria-hidden="true"
        className={dark ? "hidden h-4 w-4" : "h-4 w-4"}
      />
    </Button>
  );
}
