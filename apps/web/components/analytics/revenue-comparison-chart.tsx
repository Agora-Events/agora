"use client";

import {
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

export interface RevenueComparisonPoint {
  period: string;
  revenue: number;
  previousRevenue: number;
}

export function RevenueComparisonChart({ data }: { data: RevenueComparisonPoint[] }) {
  return (
    <section className="rounded-3xl border-2 border-black bg-white p-5 shadow-[-5px_5px_0_rgba(0,0,0,1)] sm:p-7">
      <h2 className="text-xl font-bold text-ink-deep sm:text-2xl">Revenue comparison</h2>
      <p className="mt-1 text-sm text-gray-600">Current revenue compared with the previous period.</p>
      {data.length === 0 ? (
        <ChartEmptyState message="Revenue will appear after your first paid ticket sale." />
      ) : (
        <div className="mt-6 h-72 min-w-0 sm:h-80">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data} margin={{ top: 8, right: 8, left: 0, bottom: 4 }}>
              <CartesianGrid stroke="#d1d5db" strokeDasharray="4 4" vertical={false} />
              <XAxis dataKey="period" tickLine={false} axisLine={{ stroke: "#111827" }} />
              <YAxis
                tickLine={false}
                axisLine={false}
                tickFormatter={(value) => `$${Number(value).toLocaleString()}`}
              />
              <Tooltip
                contentStyle={{
                  border: "2px solid #111827",
                  borderRadius: 12,
                  boxShadow: "-3px 3px 0 #111827",
                }}
                formatter={(value, name) => [
                  `$${Number(value).toLocaleString(undefined, { maximumFractionDigits: 2 })}`,
                  name,
                ]}
              />
              <Legend />
              <Bar dataKey="revenue" name="Current period" fill="#7c3aed" radius={[6, 6, 0, 0]} />
              <Bar
                dataKey="previousRevenue"
                name="Previous period"
                fill="#facc15"
                stroke="#111827"
                radius={[6, 6, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}
    </section>
  );
}

function ChartEmptyState({ message }: { message: string }) {
  return (
    <div className="mt-6 flex h-72 items-center justify-center rounded-2xl border-2 border-dashed border-black/20 bg-surface px-6 text-center">
      <div>
        <p className="font-bold text-ink-deep">No revenue data</p>
        <p className="mt-1 text-sm text-gray-500">{message}</p>
      </div>
    </div>
  );
}
