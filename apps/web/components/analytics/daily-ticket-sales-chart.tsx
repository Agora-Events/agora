"use client";

import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

export interface DailyTicketSalesPoint {
  date: string;
  tickets: number;
}

interface DailyTicketSalesChartProps {
  data: DailyTicketSalesPoint[];
  title?: string;
}

export function DailyTicketSalesChart({
  data,
  title = "Daily ticket sales",
}: DailyTicketSalesChartProps) {
  return (
    <section
      className="rounded-3xl border-2 border-black bg-white p-5 shadow-[-5px_5px_0_rgba(0,0,0,1)] sm:p-7"
      aria-labelledby="daily-ticket-sales-title"
    >
      <div className="mb-6">
        <h2 id="daily-ticket-sales-title" className="text-xl font-bold text-ink-deep sm:text-2xl">
          {title}
        </h2>
        <p className="mt-1 text-sm text-gray-600">
          Tickets purchased each day in the selected period.
        </p>
      </div>

      {data.length === 0 ? (
        <div
          className="flex h-72 items-center justify-center rounded-2xl border-2 border-dashed border-black/20 bg-surface px-6 text-center"
          role="status"
        >
          <div>
            <p className="font-bold text-ink-deep">No ticket sales yet</p>
            <p className="mt-1 text-sm text-gray-500">
              Daily sales will appear here after the first purchase.
            </p>
          </div>
        </div>
      ) : (
        <div className="h-72 min-w-0 sm:h-80">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data} margin={{ top: 8, right: 12, left: -18, bottom: 4 }}>
              <CartesianGrid stroke="#d1d5db" strokeDasharray="4 4" />
              <XAxis
                dataKey="date"
                tick={{ fill: "#4b5563", fontSize: 12 }}
                tickLine={false}
                axisLine={{ stroke: "#111827" }}
              />
              <YAxis
                allowDecimals={false}
                tick={{ fill: "#4b5563", fontSize: 12 }}
                tickLine={false}
                axisLine={false}
              />
              <Tooltip
                cursor={{ stroke: "#111827", strokeDasharray: "4 4" }}
                contentStyle={{
                  border: "2px solid #111827",
                  borderRadius: 12,
                  boxShadow: "-3px 3px 0 #111827",
                }}
                formatter={(value) => [Number(value).toLocaleString(), "Tickets sold"]}
              />
              <Legend />
              <Line
                type="monotone"
                dataKey="tickets"
                name="Tickets sold"
                stroke="#7c3aed"
                strokeWidth={3}
                activeDot={{ r: 6, fill: "#facc15", stroke: "#111827" }}
                dot={{ r: 4, fill: "#7c3aed", strokeWidth: 0 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}
    </section>
  );
}
