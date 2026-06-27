"use client";

import { useEffect } from "react";

export function DirtyFormGuard() {
  useEffect(() => {
    let dirty = false;

    function markDirty() {
      dirty = true;
    }

    function beforeUnload(event: BeforeUnloadEvent) {
      if (!dirty) {
        return;
      }

      event.preventDefault();
      event.returnValue = "";
    }

    document.addEventListener("input", markDirty, true);
    document.addEventListener("change", markDirty, true);
    window.addEventListener("beforeunload", beforeUnload);

    return () => {
      document.removeEventListener("input", markDirty, true);
      document.removeEventListener("change", markDirty, true);
      window.removeEventListener("beforeunload", beforeUnload);
    };
  }, []);

  return null;
}
