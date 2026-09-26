"use client";

import { useEffect, useState } from "react";
import { toast } from "sonner";

const STORAGE_KEY = "agora:followed-organizers";

type FollowButtonProps = {
  organizerId: string;
  className?: string;
};

function readFollowStore(): Record<string, boolean> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object") return {};
    return parsed as Record<string, boolean>;
  } catch {
    return {};
  }
}

function persistFollowState(organizerId: string, following: boolean): Promise<void> {
  return new Promise((resolve, reject) => {
    try {
      const store = readFollowStore();
      if (following) {
        store[organizerId] = true;
      } else {
        delete store[organizerId];
      }
      localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
      resolve();
    } catch {
      reject(new Error("Could not save follow state"));
    }
  });
}

export function FollowButton({ organizerId, className = "" }: FollowButtonProps) {
  const [following, setFollowing] = useState(false);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    try {
      const store = readFollowStore();
      setFollowing(Boolean(store[organizerId]));
    } catch {
      setFollowing(false);
    }
  }, [organizerId]);

  const handleToggle = async () => {
    if (pending) return;

    const next = !following;
    setFollowing(next);
    setPending(true);

    try {
      await persistFollowState(organizerId, next);
    } catch {
      setFollowing(!next);
      toast.error("Could not update follow state. Please try again.");
    } finally {
      setPending(false);
    }
  };

  const label = following ? "Following" : "Follow";

  return (
    <button
      type="button"
      onClick={handleToggle}
      disabled={pending}
      aria-pressed={following}
      aria-busy={pending}
      aria-label={following ? "Unfollow organizer" : "Follow organizer"}
      className={`group/follow flex w-full items-center justify-center rounded-full border-2 border-black px-5 py-2.5 text-sm font-bold transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#FDDA23] disabled:cursor-not-allowed disabled:opacity-60 ${
        following
          ? "bg-black text-white hover:bg-red-600 hover:border-red-700 hover:text-white"
          : "bg-[#FDDA23] text-black hover:bg-[#f0ce12]"
      } ${className}`}
    >
      {pending && (
        <span
          className="mr-2 inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-current/30 border-t-current"
          aria-hidden="true"
        />
      )}
      <span className={following && !pending ? "group-hover/follow:hidden group-focus-visible/follow:hidden" : undefined}>
        {label}
      </span>
      {following && !pending && (
        <span className="hidden group-hover/follow:inline group-focus-visible/follow:inline">
          Unfollow
        </span>
      )}
    </button>
  );
}
