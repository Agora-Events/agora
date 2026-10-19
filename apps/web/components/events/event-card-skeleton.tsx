import React from 'react';

export function EventCardSkeleton() {
  return (
    <div className="block w-full">
      <div className="w-full sm:max-w-147.5 shadow-[-6px_6px_0_rgba(0,0,0,1)] sm:shadow-[-9px_9px_0_rgba(0,0,0,1)] flex flex-col bg-surface pb-4.75 sm:pl-12.5 pl-4 pt-5 sm:pt-9.75 rounded-xl sm:pr-5 pr-3.75 overflow-hidden animate-pulse">
        <div className="flex gap-4.75">
          {/* Left Side: Image & Mobile Placeholders */}
          <div className="flex-shrink-0 w-[40%] sm:w-auto">
            <div className="w-full h-[112px] sm:h-[112px] rounded-lg bg-black/10" />
            
            {/* Price Label (Mobile Only) */}
            <div className="flex justify-center font-semibold sm:hidden text-[10px]/2.5 mt-4">
              <div className="w-12 h-4 bg-black/10 rounded" />
            </div>
            
            {/* View Event (Mobile Only) */}
            <div className="sm:hidden justify-center flex items-center gap-1 mt-1.5">
              <div className="w-16 h-3 bg-black/10 rounded" />
              <div className="w-4 h-4 bg-black/10 rounded-full" />
            </div>
          </div>

          {/* Right Side: Content */}
          <div className="flex flex-col grow justify-between sm:justify-start min-w-0">
            {/* Date (Desktop) */}
            <div className="font-light text-[15px]/7.5 hidden sm:block">
              <div className="w-24 h-3 bg-black/10 rounded" />
            </div>

            {/* Title */}
            <div className="mt-1 sm:mt-2.5">
              <div className="w-full h-4 bg-black/10 rounded mb-1" />
              <div className="w-3/4 h-4 bg-black/10 rounded" />
            </div>

            {/* Price Label (Desktop) */}
            <div className="max-sm:hidden pr-3 font-semibold sm:text-[13px]/3.25 mt-2 self-end">
              <div className="w-10 h-4 bg-black/10 rounded" />
            </div>

            <div>
              {/* Date (Mobile) */}
              <div className="max-sm:block hidden">
                <div className="w-20 h-2 bg-black/10 rounded" />
              </div>

              {/* Location */}
              <div className="flex items-center gap-1.25 mt-1">
                <div className="w-4 h-4 bg-black/10 rounded-full flex-shrink-0" />
                <div className="w-24 h-2 bg-black/10 rounded" />
              </div>
            </div>
          </div>
        </div>

        {/* View Event (Desktop Only) */}
        <div className="self-end hidden sm:flex mr-6 gap-1.5 mt-1.5">
          <div className="w-16 h-3 bg-black/10 rounded" />
          <div className="w-6 h-6 bg-black/10 rounded-full" />
        </div>
      </div>
    </div>
  );
}
