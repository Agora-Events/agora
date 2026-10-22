"use client";

import Image from "next/image";
import Link from "next/link";
import { useProfileDetails } from "@/hooks/use-profile-details";

const socials = [
  { name: "Instagram", icon: "/icons/instagram.svg", href: "#" },
  { name: "X", icon: "/icons/x.svg", href: "#" },
  { name: "Mail", icon: "/icons/mail.svg", href: "#" },
  { name: "LinkedIn", icon: "/icons/linkedin.svg", href: "#" },
];

export function ProfileSidebar() {
  const { details } = useProfileDetails();

  return (
    <aside className="bg-white rounded-2xl p-6 flex flex-col gap-6 shadow-sm border border-[#F0EAD6]">
      {/* Avatar */}
      <div className="flex flex-col items-center gap-3">
        <div className="w-24 h-24 rounded-full overflow-hidden border-4 border-[#FFEFD3] relative">
          <Image
            src="/images/pfp.png"
            alt="Profile photo"
            fill
            className="object-cover"
          />
        </div>
        <div className="text-center">
          <h2 className="text-xl font-semibold text-[#1A1A1A]">{details.displayName}</h2>
          <p className="text-sm text-gray-500">{details.bio || "No biography yet"}</p>
        </div>
        <h2 className="text-lg font-semibold text-ink-soft">Profile not found</h2>
        <p className="mt-2 text-sm leading-6 text-gray-500">
          This organizer profile is not available yet.
        </p>
      </div>
    </aside>
  );
}

function ProfileSkeleton() {
  return (
    <aside className="bg-white rounded-2xl p-6 flex flex-col gap-6 shadow-sm border border-border-warm animate-pulse">
      <div className="flex flex-col items-center gap-3">
        <div className="h-24 w-24 rounded-full bg-base" />
        <div className="h-5 w-36 rounded bg-base" />
        <div className="h-4 w-24 rounded bg-base" />
      </div>
      <div className="h-4 w-40 rounded bg-base" />
      <div className="flex justify-around border-t border-b border-border-warm py-4">
        <div className="h-12 w-16 rounded bg-base" />
        <div className="w-px bg-border-warm" />
        <div className="h-12 w-16 rounded bg-base" />
      </div>
    </aside>
  );
}

export function ProfileSidebar({ address = "me" }: { address?: string }) {
  const { data, error, isLoading } = useSWR<ProfileResponse, FetchError>(
    `/api/v1/profile/${encodeURIComponent(address)}`,
    fetcher,
  );

  if (isLoading) {
    return <ProfileSkeleton />;
  }

  if (error) {
    return <EmptyProfileState />;
  }

  const profile = data?.profile;
  const socialLinks = profile?.socials ?? {};
  const displayName = profile?.displayName ?? "Agora Organizer";
  const avatarUrl = profile?.avatarUrl || "/images/pfp.png";
  const joinedDate = profile?.createdAt
    ? new Intl.DateTimeFormat("en", { month: "long", year: "numeric" }).format(new Date(profile.createdAt))
    : "Recently";

  return (
    <aside className="bg-white rounded-2xl p-6 flex flex-col gap-6 shadow-sm border border-border-warm">
      <UserInfoCard
        displayName={displayName}
        address={profile?.address ?? ""}
        bio={profile?.bio}
        avatarUrl={avatarUrl}
      />

      <Link
        href="/profile/edit"
        className="self-center text-sm font-medium text-[#1A1A1A] underline underline-offset-4 hover:text-gray-600"
      >
        Edit profile
      </Link>

      {/* Joined Date */}
      <div className="flex items-center gap-2 text-sm text-gray-500">
        <Image src="/icons/calendar.svg" alt="Calendar" width={16} height={16} />
        <span>Joined {joinedDate}</span>
      </div>

      {/* Stats */}
      <div className="flex justify-around border-t border-b border-border-warm py-4">
        <div className="text-center">
          <p className="text-2xl font-bold text-ink-soft">{profile?.counts?.hosted ?? 0}</p>
          <p className="text-xs text-gray-500 mt-1">Hosted</p>
        </div>
        <div className="w-px bg-border-warm" />
        <div className="text-center">
          <p className="text-2xl font-bold text-ink-soft">{profile?.counts?.attended ?? 0}</p>
          <p className="text-xs text-gray-500 mt-1">Attended</p>
        </div>
      </div>

      {/* Socials */}
      <div className="flex justify-center gap-4">
        {socials.map(({ name, icon, href }) => (
          <a
            key={name}
            href={socialLinks[name.toLowerCase()] || href}
            aria-label={name}
            className="w-9 h-9 flex items-center justify-center rounded-full bg-base hover:bg-surface transition-colors"
          >
            <Image src={icon} alt={name} width={18} height={18} />
          </a>
        ))}
      </div>
    </aside>
  );
}
