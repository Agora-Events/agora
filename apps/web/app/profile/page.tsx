import { Navbar } from "@/components/layout/navbar";
import { UserInfoCard } from "@/components/profile/UserInfoCard";

export default function ProfilePage() {
  // Sample data - in a real app, this would come from API/auth
  const userData = {
    avatarUrl: "/images/avatar-placeholder.jpg", // This would be the user's actual avatar
    name: "John Stellar",
    role: "Designer",
    joinedDate: "Joined March 2023",
    hostedCount: 12,
    attendedCount: 28,
    instagramUrl: "https://instagram.com/johnstellar",
    twitterUrl: "https://twitter.com/johnstellar",
    mailUrl: "mailto:john@stellar.com",
    linkedinUrl: "https://linkedin.com/in/johnstellar"
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <Navbar />
      
      <div className="max-w-[1200px] mx-auto px-4 py-8">
        {/* Two-column layout */}
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Left column - Profile Sidebar */}
          <div className="lg:w-[400px] flex-shrink-0">
            <UserInfoCard {...userData} />
          </div>
          
          {/* Right column - Event lists placeholder */}
          <div className="flex-1">
            <div className="bg-white rounded-lg border border-black p-8">
              <h2 className="text-xl font-semibold mb-4">Your Events</h2>
              {/* Event lists — coming in follow-up issue */}
              <p className="text-gray-500">Event lists will be displayed here in a future update.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
