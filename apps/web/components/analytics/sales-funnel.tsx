export interface SalesFunnelData {
  pageViews: number;
  checkoutStarted: number;
  successfulPurchases: number;
}

interface SalesFunnelProps {
  data: SalesFunnelData;
  title?: string;
}

const STEPS: Array<{ key: keyof SalesFunnelData; label: string; color: string }> = [
  { key: "pageViews", label: "Page Views", color: "bg-violet-600" },
  { key: "checkoutStarted", label: "Checkout Started", color: "bg-amber-400" },
  { key: "successfulPurchases", label: "Successful Purchases", color: "bg-teal-500" },
];

export function SalesFunnel({ data, title = "Sales funnel" }: SalesFunnelProps) {
  const maximum = Math.max(data.pageViews, data.checkoutStarted, data.successfulPurchases, 1);

  return (
    <section className="rounded-3xl border-2 border-black bg-white p-5 shadow-[-5px_5px_0_rgba(0,0,0,1)] sm:p-7">
      <h2 className="text-xl font-bold text-ink-deep sm:text-2xl">{title}</h2>
      <p className="mt-1 text-sm text-gray-600">See where attendees progress or leave the purchase journey.</p>
      <ol className="mt-7 space-y-5">
        {STEPS.map((step, index) => {
          const value = data[step.key];
          const previousValue = index === 0 ? value : data[STEPS[index - 1].key];
          const conversion = previousValue > 0 ? Math.min(100, (value / previousValue) * 100) : 0;
          return (
            <li key={step.key}>
              <div className="mb-2 flex items-end justify-between gap-4">
                <span className="font-semibold text-ink-deep">{step.label}</span>
                <span className="text-right">
                  <strong className="text-xl text-ink-deep">{value.toLocaleString()}</strong>
                  {index > 0 && <small className="ml-2 text-gray-500">{conversion.toFixed(1)}%</small>}
                </span>
              </div>
              <div className="h-10 overflow-hidden rounded-xl border-2 border-black bg-surface" aria-label={`${step.label}: ${value}`}>
                <div
                  className={`h-full min-w-0 ${step.color} transition-[width]`}
                  style={{ width: value === 0 ? "0%" : `${Math.max(5, (value / maximum) * 100)}%` }}
                />
              </div>
            </li>
          );
        })}
      </ol>
    </section>
  );
}
