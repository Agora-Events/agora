import Image from "next/image";
import { Button } from "@/components/ui/button";

export function InfoSection() {
  return (
    <section className="w-full bg-ink pt-[60px] md:pt-[102px] pb-24 text-white select-none overflow-hidden">
      <div className="w-full max-w-[1240px] mx-auto px-4 flex flex-col items-center">
        {/* --- HOW AGORA WORKS --- */}

        <div className="bg-white text-black px-6 py-2 rounded-full italic text-sm mb-12">
          How agora works
        </div>

        <div className="flex flex-wrap justify-center items-start gap-6 mb-16">
          <div>
            <Image
              src="/images/How-it-works-1.png"
              alt="How it works 1"
              width={371}
              height={420}
              className="object-cover rounded-xl w-full max-w-[371px] h-auto mt-12"
            />
          </div>
          <div>
            <Image
              src="/images/How-it-works-2.png"
              alt="How it works 2"
              width={371}
              height={420}
              className="object-cover rounded-xl w-full max-w-[371px] h-auto"
            />
          </div>
          <div>
            <Image
              src="/images/How-it-works-3.png"
              alt="How it works 3"
              width={371}
              height={420}
              className="object-cover rounded-xl w-full max-w-[371px] h-auto mt-12"
            />
          </div>
        </div>

        <div className="mb-24 md:mb-48 -mt-10 md:-mt-10 relative z-10">
          <Button
            style={{ width: "215px", height: "56px" }}
            backgroundColor="bg-accent"
            textColor="text-black"
            shadowColor="white"
          >
            <span className="font-semibold">Discover Events</span>
            <Image src="/icons/earth.svg" alt="Earth" width={20} height={20} />
          </Button>
        </div>

        {/* --- ABOUT US --- */}

        <div className="flex flex-col items-center w-full">
          {/* Centered Pill */}
          <div className="bg-white text-black px-4 py-1.5 rounded-full italic text-sm mb-12">
            What is agora about ?
          </div>

          <div className="w-full flex flex-col lg:flex-row items-center gap-12 lg:gap-30">
            {/* Left: Image */}
            <div className="w-full lg:w-auto flex justify-center lg:justify-start">
              <div className="relative w-[300px] h-[296px] md:w-[352px] md:h-[348px]">
                <Image
                  src="/images/AboutUs.png"
                  alt="About Us"
                  fill
                  className="object-contain"
                />
              </div>
            </div>

            {/* Right: Content */}
            <div className="flex-1 text-left px-4 md:px-0">
              {/* Heading */}
              <h2 className="font-bold italic text-[28px] md:text-[32px] leading-[32px] mt-8 mb-6 text-center lg:text-left">
                About Us
              </h2>

              {/* Paragraph */}
              <div className="text-[16px] md:text-[20px] leading-[26px] md:leading-[30px] font-normal font-sans text-white space-y-6 text-center lg:text-left">
                <p>
                  Agora is an event and ticketing platform built for organizers,
                  creators, and communities to create events, sell tickets, and
                  manage attendees with ease.
                </p>
                <p>
                  We handle ticketing, payments, and infrastructure so
                  organizers can focus on bringing people together. Attendees
                  can follow organizers, discover upcoming events, and stay
                  connected beyond a single event.
                </p>
                <p>
                  Built on Stellar, Agora enables fast, low-cost, borderless
                  payments using USDC, enabling fast, reliable payouts for
                  organizers.
                </p>
                <p>Agora is where events grow into communities.</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
