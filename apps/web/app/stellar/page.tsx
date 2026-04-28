import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import Image from "next/image";

export default function StellarPage() {
  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />
      <div className="flex-1 flex flex-col items-center justify-center p-4 text-center">
        <div className="bg-[#060606] p-8 rounded-3xl mb-8">
           <Image
            src="/icons/stellar-logo.svg"
            alt="Stellar"
            width={64}
            height={64}
            className="mb-4 mx-auto"
          />
          <h1 className="text-4xl font-bold text-white mb-4 italic">Stellar Ecosystem</h1>
          <p className="text-xl text-gray-400 mb-8 max-w-md">
            Agora is powered by Stellar. Secure, fast, and low-cost transactions for global events.
          </p>
          <Link href="/">
            <Button backgroundColor="bg-[#FDDA23]" textColor="text-black" shadowColor="rgba(253,218,35,0.4)">
              Learn More About Stellar
            </Button>
          </Link>
        </div>
      </div>
      <Footer />
    </main>
  );
}
