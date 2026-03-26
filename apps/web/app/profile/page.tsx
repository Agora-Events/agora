import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { ProfileSidebar } from "@/components/profile/profile-sidebar";

export default function ProfilePage() {
  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <div className="flex-1 w-full max-w-6xl mx-auto px-4 py-10">
        <div className="flex flex-col md:flex-row gap-8 items-start">
          {/* Sidebar — ~28% on desktop, full-width stacked on mobile */}
          <div className="w-full md:w-[28%] md:sticky md:top-24">
            <ProfileSidebar />
          </div>

          {/* Main content area — placeholder for future event lists */}
          <div className="flex-1 min-h-[400px] bg-white rounded-2xl border border-[#F0EAD6] shadow-sm flex items-center justify-center">
            <p className="text-gray-400 text-sm">Events will appear here</p>
          </div>
        </div>
      </div>
      <Footer />
    </main>
  );
}
