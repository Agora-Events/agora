"use client";

import { useEffect } from "react";
import { usePathname, useSearchParams } from "next/navigation";
import { captureAttribution } from "@/utils/attribution";

export function AttributionCapture() {
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const query = searchParams.toString();

  useEffect(() => {
    captureAttribution(new URLSearchParams(query));
  }, [pathname, query]);

  return null;
}
