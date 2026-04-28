"use client";

import { useEffect } from "react";
import NProgress from "nprogress";
import "nprogress/nprogress.css";
import { usePathname, useSearchParams } from "next/navigation";

export default function LoadingBar() {
  const pathname = usePathname();
  const searchParams = useSearchParams();

  useEffect(() => {
    // Configure NProgress
    NProgress.configure({
      showSpinner: false,
      trickleSpeed: 200,
      minimum: 0.08,
    });
  }, []);

  useEffect(() => {
    // This effect runs whenever the pathname or searchParams change
    // We start the progress when navigation starts and stop it when it ends
    // In App Router, we don't have direct events for "navigation started",
    // but the effect running means the navigation has "happened" (client-side).
    // To properly capture the "loading" state during data fetching,
    // this component should be used in conjunction with Suspense.
    NProgress.done();

    return () => {
      NProgress.start();
    };
  }, [pathname, searchParams]);

  return null;
}
