"use client";

import { useEffect, useRef } from "react";

export function DirtyFormGuard() {
  const markerRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const form = markerRef.current?.closest("form");
    if (!form) {
      return;
    }

    let dirty = false;

    function markDirty() {
      dirty = true;
    }

    function resetDirty() {
      dirty = false;
    }

    function beforeUnload(event: BeforeUnloadEvent) {
      if (!dirty) {
        return;
      }

      event.preventDefault();
      event.returnValue = "";
    }

    function beforeDocumentClick(event: MouseEvent) {
      if (!dirty || event.defaultPrevented) {
        return;
      }

      const target = event.target;
      if (!(target instanceof Element)) {
        return;
      }

      const link = target.closest("a[href]");
      if (!(link instanceof HTMLAnchorElement)) {
        return;
      }

      if (
        link.target !== "" ||
        link.hasAttribute("download") ||
        event.metaKey ||
        event.ctrlKey ||
        event.shiftKey ||
        event.altKey
      ) {
        return;
      }

      if (
        !window.confirm(
          "You have unsaved library edits. Leave this item without saving?",
        )
      ) {
        event.preventDefault();
        event.stopPropagation();
      }
    }

    form.addEventListener("input", markDirty, true);
    form.addEventListener("change", markDirty, true);
    form.addEventListener("submit", resetDirty);
    document.addEventListener("click", beforeDocumentClick, true);
    window.addEventListener("beforeunload", beforeUnload);

    return () => {
      form.removeEventListener("input", markDirty, true);
      form.removeEventListener("change", markDirty, true);
      form.removeEventListener("submit", resetDirty);
      document.removeEventListener("click", beforeDocumentClick, true);
      window.removeEventListener("beforeunload", beforeUnload);
    };
  }, []);

  return <span ref={markerRef} hidden data-dirty-form-guard />;
}
