import CreateEventForm from "@/components/events/create-event-form";
import { Camera } from "lucide-react";

export default function CreateEventPage() {
  return (
    <div className="container mx-auto px-4 py-8 max-w-7xl pb-24 bg-[#FAF9F6] min-h-screen">
      <h1 className="text-4xl md:text-5xl font-bold mb-8 italic">Create your Event</h1>
      <div className="flex flex-col md:flex-row gap-8">
        
        {/* Left Column: Cover Photo Upload Placeholder */}
        <div className="w-full md:w-[400px] shrink-0">
          <div className="relative w-full aspect-[4/5] bg-gradient-to-br from-[#1b8c8c] to-[#601a58] rounded-2xl flex flex-col items-center justify-center p-6 border-4 border-transparent hover:border-black border-dashed transition-all cursor-pointer overflow-hidden shadow-sm">
            {/* Overlay decoration for gradient */}
            <div className="absolute inset-0 bg-black/5" />
            
            {/* Ticket style text overlays */}
            <div className="z-10 w-full flex flex-col gap-4 items-center justify-center mb-10 translate-x-4 -rotate-[15deg]">
               <div className="border hover:bg-white/10 transition-colors border-white text-white rounded-[32px] px-8 py-2 text-4xl font-light backdrop-blur-sm">
                 You&apos;re
               </div>
               <div className="border hover:bg-white/10 transition-colors border-white text-white rounded-[32px] px-10 py-2 text-4xl font-light -translate-x-8 backdrop-blur-sm">
                 Invited
               </div>
            </div>
            
            {/* Camera icon button */}
            <div className="absolute bottom-4 right-4 bg-white/20 p-3 rounded-full backdrop-blur-sm shadow-sm cursor-pointer hover:bg-white/30 transition-colors border border-white/30 text-white">
              <Camera className="w-6 h-6" />
            </div>
          </div>
        </div>

        {/* Right Column: Complete Form */}
        <div className="flex-1">
          <CreateEventForm />
        </div>

      </div>
    </div>
  );
}
