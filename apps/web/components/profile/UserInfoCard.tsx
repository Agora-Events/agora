"use client";

import Image from "next/image";
import { WalletAddress } from "@/components/ui/wallet-address";

type UserInfoCardProps = {
  displayName: string;
  address: string;
  bio?: string | null;
  avatarUrl?: string | null;
};

export function UserInfoCard({
  displayName,
  address,
  bio,
  avatarUrl,
}: UserInfoCardProps) {
  return (
    <div className="flex flex-col items-center gap-3">
      <div className="relative h-24 w-24 overflow-hidden rounded-full border-4 border-surface">
        <Image
          src={avatarUrl || "/images/pfp.png"}
          alt={`${displayName} profile photo`}
          fill
          className="object-cover"
        />
      </div>
      <div className="text-center min-w-0 w-full">
        <h2 className="text-xl font-semibold text-ink-soft">{displayName}</h2>
        <p className="text-sm text-gray-500">{bio || "Agora community member"}</p>
      </div>
      {address && address !== "me" && (
        <WalletAddress address={address} className="text-gray-700" />
      )}
    </div>
  );
}
