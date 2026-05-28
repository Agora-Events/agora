import Image from "next/image";

const socials = [
  { name: "Instagram", icon: "/icons/instagram.svg", href: "#" },
  { name: "X", icon: "/icons/x.svg", href: "#" },
  { name: "Mail", icon: "/icons/mail.svg", href: "#" },
  { name: "LinkedIn", icon: "/icons/linkedin.svg", href: "#" },
];

export function ProfileSidebar() {
  return (
    <aside className="bg-white rounded-2xl p-6 flex flex-col gap-6 shadow-sm border border-border-warm">
      {/* Avatar */}
      <div className="flex flex-col items-center gap-3">
        <div className="w-24 h-24 rounded-full overflow-hidden border-4 border-surface relative">
          <Image
            src="/images/pfp.png"
            alt="Profile photo"
            fill
            className="object-cover"
          />
        </div>
        <div className="text-center">
          <h2 className="text-xl font-semibold text-ink-soft">John Stellar</h2>
          <p className="text-sm text-gray-500">Designer</p>
        </div>
      </div>

      {/* Joined Date */}
      <div className="flex items-center gap-2 text-sm text-gray-500">
        <Image src="/icons/calendar.svg" alt="Calendar" width={16} height={16} />
        <span>Joined January 2024</span>
      </div>

      {/* Stats */}
      <div className="flex justify-around border-t border-b border-border-warm py-4">
        <div className="text-center">
          <p className="text-2xl font-bold text-ink-soft">12</p>
          <p className="text-xs text-gray-500 mt-1">Hosted</p>
        </div>
        <div className="w-px bg-border-warm" />
        <div className="text-center">
          <p className="text-2xl font-bold text-ink-soft">34</p>
          <p className="text-xs text-gray-500 mt-1">Attended</p>
        </div>
      </div>

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
