import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import Image from "next/image";

export default function StellarPage() {
  return (
    <main className="flex flex-col min-h-screen bg-base">
      <Navbar />
      <div className="flex-1 max-w-[1221px] w-full mx-auto px-4 lg:px-0 py-12 md:py-20 flex flex-col gap-16">
        
        {/* Hero Section */}
        <div className="bg-ink p-10 md:p-16 rounded-[40px] flex flex-col items-center justify-center text-center shadow-[-8px_8px_0_rgba(0,0,0,1)]">
          <Image
            src="/icons/stellar-logo.svg"
            alt="Stellar"
            width={96}
            height={96}
            className="mb-6 mx-auto"
          />
          <h1 className="text-4xl md:text-6xl font-extrabold text-white mb-6 italic">Powered by Stellar</h1>
          <p className="text-xl md:text-2xl text-gray-300 mb-10 max-w-2xl font-light">
            Agora runs on the Stellar network, providing lightning-fast, ultra-low-cost, and secure transactions for global events.
          </p>
          <div className="flex flex-col sm:flex-row gap-4">
            <Link href="/create-event">
              <Button backgroundColor="bg-accent" textColor="text-black" shadowColor="rgba(253,218,35,0.4)" className="px-8 py-6 text-lg font-bold">
                Start Hosting
              </Button>
            </Link>
            <a href="https://stellar.org" target="_blank" rel="noreferrer">
              <Button backgroundColor="bg-white" textColor="text-black" shadowColor="rgba(0,0,0,1)" className="px-8 py-6 text-lg font-bold">
                Stellar Docs
              </Button>
            </a>
          </div>
        </div>

        {/* Why Stellar? */}
        <section className="flex flex-col md:flex-row gap-10">
          <div className="flex-1 bg-surface p-10 rounded-3xl border-2 border-black shadow-[-6px_6px_0_rgba(0,0,0,1)]">
            <h2 className="text-3xl font-bold mb-6 italic">Why Stellar?</h2>
            <ul className="space-y-6 text-lg">
              <li className="flex items-start gap-4">
                <div className="bg-success w-10 h-10 rounded-full flex items-center justify-center border-2 border-black shrink-0">🚀</div>
                <div>
                  <h3 className="font-bold text-xl mb-1">Blazing Fast</h3>
                  <p className="text-ink/80">Transactions settle in 3-5 seconds, meaning your tickets are issued instantly without any waiting.</p>
                </div>
              </li>
              <li className="flex items-start gap-4">
                <div className="bg-accent w-10 h-10 rounded-full flex items-center justify-center border-2 border-black shrink-0">💸</div>
                <div>
                  <h3 className="font-bold text-xl mb-1">Ultra-Low Fees</h3>
                  <p className="text-ink/80">Stellar transaction fees are fractions of a cent, ensuring that event organizers and attendees keep more of their money.</p>
                </div>
              </li>
              <li className="flex items-start gap-4">
                <div className="bg-[#E8D5F7] w-10 h-10 rounded-full flex items-center justify-center border-2 border-black shrink-0">🌍</div>
                <div>
                  <h3 className="font-bold text-xl mb-1">Global Access</h3>
                  <p className="text-ink/80">Borderless payments allow anyone anywhere in the world to buy or sell tickets seamlessly.</p>
                </div>
              </li>
            </ul>
          </div>
          
          <div className="flex-1 flex flex-col gap-10">
            {/* USDC Payments */}
            <div className="bg-white p-10 rounded-3xl border-2 border-black shadow-[-6px_6px_0_rgba(0,0,0,1)]">
              <h2 className="text-3xl font-bold mb-6 italic">USDC on Agora</h2>
              <p className="text-lg text-ink/80 mb-6">
                All ticket purchases and payouts on Agora are processed in USDC on the Stellar network. USDC is a fully reserved stablecoin pegged to the US Dollar.
              </p>
              <Link href="https://developers.stellar.org/docs" target="_blank">
                <Button variant="outline" className="w-full justify-between items-center text-lg font-bold border-2">
                  Read the Payment Guide <span>→</span>
                </Button>
              </Link>
            </div>

            {/* Wallet Setup */}
            <div className="bg-[#D5F7E8] p-10 rounded-3xl border-2 border-black shadow-[-6px_6px_0_rgba(0,0,0,1)]">
              <h2 className="text-3xl font-bold mb-6 italic">Wallet Setup</h2>
              <p className="text-lg text-ink/80 mb-6">
                To interact with Agora, you'll need a Stellar-compatible wallet like Freighter or Lobstr. Connect your wallet to easily manage your tickets and events.
              </p>
              <a href="https://freighter.app" target="_blank" rel="noreferrer">
                <Button variant="primary" className="w-full text-lg font-bold">
                  Get Freighter Wallet
                </Button>
              </a>
            </div>
          </div>
        </section>

        {/* FAQ Section */}
        <section className="bg-white p-10 md:p-16 rounded-[40px] border-2 border-black shadow-[-8px_8px_0_rgba(0,0,0,1)]">
          <h2 className="text-4xl font-bold mb-10 text-center italic">Frequently Asked Questions</h2>
          <div className="grid md:grid-cols-2 gap-8">
            <div className="bg-surface p-8 rounded-2xl border border-black shadow-[-4px_4px_0_rgba(0,0,0,1)]">
              <h3 className="text-xl font-bold mb-3">Do I need cryptocurrency to use Agora?</h3>
              <p className="text-ink/80">While Agora uses crypto rails behind the scenes, we make it easy to pay with stablecoins like USDC which are tied 1:1 to the US Dollar.</p>
            </div>
            <div className="bg-surface p-8 rounded-2xl border border-black shadow-[-4px_4px_0_rgba(0,0,0,1)]">
              <h3 className="text-xl font-bold mb-3">How do I fund my wallet?</h3>
              <p className="text-ink/80">You can easily convert fiat (like USD or EUR) into USDC directly through on-ramps available in most Stellar wallets, or send USDC from a crypto exchange.</p>
            </div>
            <div className="bg-surface p-8 rounded-2xl border border-black shadow-[-4px_4px_0_rgba(0,0,0,1)]">
              <h3 className="text-xl font-bold mb-3">Are my tickets NFTs?</h3>
              <p className="text-ink/80">Yes! Every ticket issued on Agora is a unique digital asset on the Stellar blockchain, ensuring authenticity and preventing fraud.</p>
            </div>
            <div className="bg-surface p-8 rounded-2xl border border-black shadow-[-4px_4px_0_rgba(0,0,0,1)]">
              <h3 className="text-xl font-bold mb-3">What happens if an event is canceled?</h3>
              <p className="text-ink/80">Smart contracts enable seamless and automated refunds directly back to your wallet if the organizer cancels the event.</p>
            </div>
          </div>
        </section>

      </div>
      <Footer />
    </main>
  );
}
