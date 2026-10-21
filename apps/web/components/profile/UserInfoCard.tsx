import Image from "next/image";

interface UserInfoCardProps {
  avatarUrl: string;
  name: string;
  role: string;
  joinedDate: string;
  hostedCount: number;
  attendedCount: number;
  instagramUrl?: string;
  twitterUrl?: string;
  mailUrl?: string;
  linkedinUrl?: string;
}

export function UserInfoCard({
  avatarUrl,
  name,
  role,
  joinedDate,
  hostedCount,
  attendedCount,
  instagramUrl,
  twitterUrl,
  mailUrl,
  linkedinUrl,
}: UserInfoCardProps) {
  return (
    <div className="bg-white rounded-lg border border-black p-6">
      {/* Avatar */}
      <div className="flex justify-center mb-6">
        <div className="w-24 h-24 rounded-full overflow-hidden border-2 border-black">
          <Image
            src={avatarUrl}
            alt={name}
            width={96}
            height={96}
            className="w-full h-full object-cover"
          />
        </div>
      </div>

      {/* Name and Role */}
      <div className="text-center mb-6">
        <h1 className="text-2xl font-semibold text-black mb-1">{name}</h1>
        <p className="text-gray-600">{role}</p>
      </div>

      {/* Joined Date */}
      <div className="flex items-center gap-3 mb-6 text-gray-600">
        <div className="w-5 h-5 flex items-center justify-center">
          {/* Calendar icon placeholder - using SVG until icon is available */}
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <rect
              x="3"
              y="4"
              width="18"
              height="18"
              rx="2"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path
              d="M16 2V6M8 2V6M3 10H21"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
        </div>
        <span className="text-sm">{joinedDate}</span>
      </div>

      {/* Stats Row */}
      <div className="flex justify-around mb-6 py-4 border-t border-b border-gray-200">
        <div className="text-center">
          <div className="text-2xl font-semibold text-black">{hostedCount}</div>
          <div className="text-sm text-gray-600">Hosted</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-semibold text-black">{attendedCount}</div>
          <div className="text-sm text-gray-600">Attended</div>
        </div>
      </div>

      {/* Social Icons Row */}
      <div className="flex justify-center gap-4">
        {instagramUrl && (
          <a
            href={instagramUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="w-10 h-10 rounded-full bg-gray-100 flex items-center justify-center hover:bg-gray-200 transition-colors"
            aria-label="Instagram"
          >
            {/* Instagram icon placeholder */}
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <rect
                x="2"
                y="2"
                width="20"
                height="20"
                rx="5"
                stroke="currentColor"
                strokeWidth="2"
              />
              <circle
                cx="12"
                cy="12"
                r="4"
                stroke="currentColor"
                strokeWidth="2"
              />
              <circle cx="18" cy="6" r="1" fill="currentColor" />
            </svg>
          </a>
        )}

        {twitterUrl && (
          <a
            href={twitterUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="w-10 h-10 rounded-full bg-gray-100 flex items-center justify-center hover:bg-gray-200 transition-colors"
            aria-label="X (Twitter)"
          >
            {/* X/Twitter icon placeholder */}
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M4 4L11 12L4 20M20 4H13L20 12V4Z"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <path
                d="M13 12L20 20M13 4H20"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </a>
        )}

        {mailUrl && (
          <a
            href={mailUrl}
            className="w-10 h-10 rounded-full bg-gray-100 flex items-center justify-center hover:bg-gray-200 transition-colors"
            aria-label="Email"
          >
            {/* Mail icon placeholder */}
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <rect
                x="3"
                y="5"
                width="18"
                height="14"
                rx="2"
                stroke="currentColor"
                strokeWidth="2"
              />
              <path
                d="M3 7L12 13L21 7"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </a>
        )}

        {linkedinUrl && (
          <a
            href={linkedinUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="w-10 h-10 rounded-full bg-gray-100 flex items-center justify-center hover:bg-gray-200 transition-colors"
            aria-label="LinkedIn"
          >
            {/* LinkedIn icon placeholder */}
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <rect
                x="3"
                y="3"
                width="18"
                height="18"
                rx="2"
                stroke="currentColor"
                strokeWidth="2"
              />
              <path
                d="M8 11V16M8 8V8.01M12 11V16M16 11V16"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
            </svg>
          </a>
        )}
      </div>
    </div>
  );
}
