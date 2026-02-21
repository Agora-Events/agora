import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";

export default function EventDetailPage({
  params,
}: {
  params: { id: string };
}) {
  return (
    <main className="flex flex-col min-h-screen bg-[#FFFBE9]">
      <Navbar />

      <div className="flex-1 w-full max-w-[1221px] mx-auto px-4 py-8">
        <h1 className="text-2xl font-bold mb-4">Event Details: {params.id}</h1>
        <p className="text-gray-600 max-w-2xl">
          Contributors: Implement the Event Detail layout here based on the
          Figma designs. You will need to fetch the mock event data based on the
          ID in the URL, and conditionally render the Registration/Payment box
          depending on if the event is Free or Paid!
        </p>

        {/* TODO: Add Image, Title, Location, Map, and Registration Box components here */}
      </div>

      <Footer />
    </main>
  );
}
