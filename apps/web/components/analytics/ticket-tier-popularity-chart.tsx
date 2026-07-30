"use client";

import { Cell, Legend, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";

export interface TicketTierPopularityPoint {
  name: string;
  tickets: number;
}

const COLORS = ["#7c3aed", "#facc15", "#14b8a6", "#f97316", "#ec4899", "#3b82f6"];

export function TicketTierPopularityChart({ data }: { data: TicketTierPopularityPoint[] }) {
  return (
    <section className="rounded-3xl border-2 border-black bg-white p-5 shadow-[-5px_5px_0_rgba(0,0,0,1)] sm:p-7">
      <h2 className="text-xl font-bold text-ink-deep sm:text-2xl">Ticket tier popularity</h2>
      <p className="mt-1 text-sm text-gray-600">Share of tickets sold across your available tiers.</p>
      {data.length === 0 || data.every((tier) => tier.tickets === 0) ? (
        <div className="mt-6 flex h-72 items-center justify-center rounded-2xl border-2 border-dashed border-black/20 bg-surface px-6 text-center">
          <div>
            <p className="font-bold text-ink-deep">No ticket tier data</p>
            <p className="mt-1 text-sm text-gray-500">Tier popularity will appear after tickets are sold.</p>
          </div>
        </div>
      ) : (
        <div className="mt-6 h-72 min-w-0 sm:h-80">
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie
                data={data}
                dataKey="tickets"
                nameKey="name"
                cx="50%"
                cy="45%"
                innerRadius="42%"
                outerRadius="72%"
                paddingAngle={2}
                stroke="#111827"
                strokeWidth={1.5}
              >
                {data.map((tier, index) => (
                  <Cell key={tier.name} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip
                contentStyle={{
                  border: "2px solid #111827",
                  borderRadius: 12,
                  boxShadow: "-3px 3px 0 #111827",
                }}
                formatter={(value) => [`${Number(value).toLocaleString()} tickets`, "Sold"]}
              />
              <Legend verticalAlign="bottom" iconType="circle" />
            </PieChart>
          </ResponsiveContainer>
        </div>
      )}
    </section>
  );
}
