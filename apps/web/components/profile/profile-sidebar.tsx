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
      {/* Avatar & Info */}
      <div className="flex flex-col items-center gap-3">
        <div className="w-24 h-24 rounded-full overflow-hidden border-4 border-[#FFEFD3] relative">
          <Image src="/images/pfp.png" alt="Profile photo" fill className="object-cover" />
        </div>
        <div className="text-center">
          <h2 className="text-xl font-semibold text-[#1A1A1A]">
            {details.displayName || "Agora Organizer"}
          </h2>
          <p className="text-sm text-gray-500">{details.bio || "No biography yet"}</p>
        </div>
      </div>

      <Link
        href="/profile/edit"
        className="self-center text-sm font-medium text-[#1A1A1A] underline underline-offset-4 hover:text-gray-600"
      >
        Edit profile
      </Link>

      {/* Socials */}
      <div className="flex justify-center gap-4">
        {socials.map(({ name, icon, href }) => (
          <a
            key={name}
            href={href}
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
