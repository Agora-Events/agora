import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function OrganizersPage() {
  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <div className="flex-1 flex flex-col items-center justify-center p-4 text-center">
        <h1 className="text-4xl font-bold mb-4">Organizers</h1>
        <p className="text-xl text-gray-600 mb-8 max-w-md">
          Discover top event organizers on Agora. This page is currently under development.
        </p>
        <Link href="/">
          <Button backgroundColor="bg-black" textColor="text-white" shadowColor="rgba(0,0,0,0.2)">
            Back to Home
          </Button>
        </Link>
      </div>
      <Footer />
    </main>
  );
}
