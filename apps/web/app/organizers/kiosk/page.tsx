"use client";

import { useAuth } from "@/hooks/useAuth";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState } from "react";

type ScanOutcome =
  | {
      type: "success";
      attendee: string;
      ticketTier: string;
      reason?: never;
    }
  | {
      type: "error";
      attendee?: never;
      ticketTier?: never;
      reason: string;
    };

const OVERLAY_DURATION_MS = 3000;
const EXIT_PIN = "2639";
const DEMO_SCANS: ScanOutcome[] = [
  { type: "success", attendee: "Ava Thompson", ticketTier: "VIP" },
  { type: "error", reason: "Already checked in" },
  { type: "success", attendee: "Marcus Lee", ticketTier: "General Admission" },
  { type: "error", reason: "Wrong event" },
  { type: "success", attendee: "Priya Shah", ticketTier: "VIP" },
];

export default function OrganizerKioskPage() {
  const router = useRouter();
  const { isAuthenticated, isLoading } = useAuth();
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const [scanOutcome, setScanOutcome] = useState<ScanOutcome | null>(null);
  const [showOverlay, setShowOverlay] = useState(false);
  const [isExitOpen, setIsExitOpen] = useState(false);
  const [pin, setPin] = useState("");
  const [pinError, setPinError] = useState("");
  const [cameraError, setCameraError] = useState<string | null>(null);

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.replace("/auth");
    }
  }, [isAuthenticated, isLoading, router]);

  useEffect(() => {
    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
      return "";
    };

    const handlePopState = () => {
      if (!isExitOpen) {
        window.history.pushState(null, "", window.location.href);
        setIsExitOpen(true);
      }
    };

    window.addEventListener("beforeunload", handleBeforeUnload);
    window.history.pushState(null, "", window.location.href);
    window.addEventListener("popstate", handlePopState);

    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
      window.removeEventListener("popstate", handlePopState);
    };
  }, [isExitOpen]);

  useEffect(() => {
    let cancelled = false;

    const openCamera = async () => {
      if (!navigator.mediaDevices?.getUserMedia) {
        setCameraError("Camera not supported on this device.");
        return;
      }

      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: {
            facingMode: "environment",
            width: { ideal: 1280 },
            height: { ideal: 720 },
          },
          audio: false,
        });

        if (cancelled) {
          stream.getTracks().forEach((track) => track.stop());
          return;
        }

        streamRef.current = stream;
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          await videoRef.current.play();
        }
      } catch {
        setCameraError("Unable to access the camera. Please check permissions.");
      }
    };

    void openCamera();

    return () => {
      cancelled = true;
      if (streamRef.current) {
        streamRef.current.getTracks().forEach((track) => track.stop());
        streamRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    if (!isAuthenticated) return;

    let timeoutId: number | undefined;
    let scanTimer: number | undefined;
    let scanIndex = 0;

    const triggerNextScan = () => {
      const nextScan = DEMO_SCANS[scanIndex % DEMO_SCANS.length];
      scanIndex += 1;
      setScanOutcome(nextScan);
      setShowOverlay(true);

      timeoutId = window.setTimeout(() => {
        setShowOverlay(false);
        setScanOutcome(null);
        scanTimer = window.setTimeout(triggerNextScan, 250);
      }, OVERLAY_DURATION_MS);
    };

    scanTimer = window.setTimeout(triggerNextScan, 1200);

    return () => {
      if (timeoutId) window.clearTimeout(timeoutId);
      if (scanTimer) window.clearTimeout(scanTimer);
    };
  }, [isAuthenticated]);

  const handleExit = () => {
    setIsExitOpen(true);
    setPinError("");
  };

  const handlePinSubmit = () => {
    if (pin === EXIT_PIN) {
      setIsExitOpen(false);
      router.replace("/organizers");
      return;
    }

    setPinError("Incorrect PIN");
  };

  if (isLoading || !isAuthenticated) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-black text-white">
        <div className="text-center">
          <div className="mx-auto h-12 w-12 animate-spin rounded-full border-4 border-white/20 border-t-accent" />
          <p className="mt-4 text-lg font-medium">Loading kiosk…</p>
        </div>
      </main>
    );
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-[#0b1020] p-3 text-white">
      <div className="relative h-[768px] w-full max-w-[1024px] overflow-hidden rounded-[28px] border border-white/10 bg-black shadow-[0_40px_120px_rgba(0,0,0,0.7)]">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_rgba(0,0,0,0.12),_rgba(0,0,0,0.75))]" />

        <div className="absolute inset-0">
          <video
            ref={videoRef}
            autoPlay
            muted
            playsInline
            className="h-full w-full object-cover"
          />
        </div>

        <div className="absolute inset-0 flex flex-col justify-between p-5 md:p-7">
          <div className="flex items-center justify-between">
            <div className="rounded-full border border-white/20 bg-black/30 px-4 py-2 text-xs font-semibold uppercase tracking-[0.18em] text-white/80 backdrop-blur-sm">
              Self-check-in
            </div>
            <button
              type="button"
              onClick={handleExit}
              className="rounded-full border border-white/20 bg-black/40 px-4 py-2 text-sm font-semibold text-white backdrop-blur-sm transition hover:bg-black/60"
            >
              Exit Kiosk
            </button>
          </div>

          <div className="pointer-events-none absolute inset-0 flex items-center justify-center px-12">
            <div className="h-[60%] w-[70%] rounded-[28px] border-[3px] border-white/60 border-dashed bg-white/5" />
          </div>

          <div className="relative z-10 flex items-end justify-between">
            <div className="rounded-full border border-white/20 bg-black/30 px-4 py-2 text-xs text-white/80 backdrop-blur-sm">
              Ready for scan
            </div>
            <div className="rounded-full border border-white/20 bg-black/30 px-4 py-2 text-xs text-white/80 backdrop-blur-sm">
              Landscape mode
            </div>
          </div>
        </div>

        {cameraError ? (
          <div className="absolute inset-0 z-20 flex items-center justify-center bg-black/80 p-8 text-center">
            <div>
              <p className="text-xl font-semibold text-red-400">Camera unavailable</p>
              <p className="mt-3 text-sm text-white/80">{cameraError}</p>
            </div>
          </div>
        ) : null}

        {showOverlay && scanOutcome ? (
          <div
            aria-live="polite"
            className={`absolute inset-0 z-30 flex items-center justify-center transition-opacity duration-300 ${
              scanOutcome.type === "success" ? "bg-green-600/90" : "bg-red-600/90"
            }`}
          >
            <div className="text-center text-white">
              <div className="mb-6 text-6xl" aria-hidden="true">
                {scanOutcome.type === "success" ? "✓" : "!"}
              </div>
              {scanOutcome.type === "success" ? (
                <>
                  <p className="text-3xl font-black uppercase tracking-[0.12em]">Entry approved</p>
                  <h2 className="mt-3 text-5xl font-bold">{scanOutcome.attendee}</h2>
                  <p className="mt-3 text-2xl text-white/90">{scanOutcome.ticketTier}</p>
                </>
              ) : (
                <>
                  <p className="text-3xl font-black uppercase tracking-[0.12em]">Access denied</p>
                  <p className="mt-4 text-3xl font-semibold">{scanOutcome.reason}</p>
                </>
              )}
            </div>
          </div>
        ) : null}

        {isExitOpen ? (
          <div className="absolute inset-0 z-40 flex items-center justify-center bg-black/75 p-6">
            <div className="w-full max-w-md rounded-[24px] border border-white/10 bg-[#101827] p-6 shadow-2xl">
              <p className="text-xs uppercase tracking-[0.2em] text-white/60">Exit kiosk</p>
              <h3 className="mt-3 text-2xl font-bold">Unlock kiosk</h3>
              <input
                type="password"
                value={pin}
                onChange={(event) => {
                  setPin(event.target.value);
                  if (pinError) setPinError("");
                }}
                inputMode="numeric"
                aria-label="Kiosk exit PIN"
                placeholder="Enter PIN"
                className="mt-4 w-full rounded-xl border border-white/15 bg-white/5 px-4 py-3 text-base text-white outline-none placeholder:text-white/40 focus:border-accent"
              />
              {pinError ? <p className="mt-2 text-sm text-red-400">{pinError}</p> : null}
              <div className="mt-5 flex gap-3">
                <button
                  type="button"
                  onClick={() => {
                    setIsExitOpen(false);
                    setPin("");
                    setPinError("");
                  }}
                  className="flex-1 rounded-xl border border-white/15 bg-white/5 px-4 py-3 font-semibold text-white"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onClick={handlePinSubmit}
                  className="flex-1 rounded-xl bg-accent px-4 py-3 font-bold text-black"
                >
                  Unlock
                </button>
              </div>
            </div>
          </div>
        ) : null}
      </div>
    </main>
  );
}
