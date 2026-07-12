"use client";

import { useRouter } from "next/navigation";
import { useEffect } from "react";

export function EpisodeGeneratingRefresh() {
  const router = useRouter();

  useEffect(() => {
    const interval = window.setInterval(() => router.refresh(), 1_500);
    return () => window.clearInterval(interval);
  }, [router]);

  return null;
}
