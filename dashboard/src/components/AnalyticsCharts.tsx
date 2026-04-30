import {
  CartesianGrid,
  Cell,
  Line,
  LineChart,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';

const COLORS = ['#22c55e', '#3b82f6', '#f59e0b', '#ef4444'];

type RevenuePoint = { date: string; revenue: number };
type MethodPoint = { name: string; value: number };

export function AnalyticsCharts({
  revenueData,
  methodData,
  formatCurrency,
}: {
  revenueData: RevenuePoint[];
  methodData: MethodPoint[];
  formatCurrency: (n: number) => string;
}) {
  return (
    <div className="charts-grid">
      <div className="chart-card">
        <h3>Revenue Trend</h3>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={revenueData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#d5c9af" />
            <XAxis dataKey="date" stroke="#6b6862" />
            <YAxis stroke="#6b6862" tickFormatter={(v) => `₹${v / 1000}k`} />
            <Tooltip
              contentStyle={{ backgroundColor: '#fff9ec', border: '1px solid #d5c9af', borderRadius: '8px' }}
              formatter={(val: number) => [formatCurrency(val), 'Revenue']}
            />
            <Line type="monotone" dataKey="revenue" stroke="#eb001b" strokeWidth={2.5} dot={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>

      <div className="chart-card">
        <h3>Payment Methods</h3>
        <ResponsiveContainer width="100%" height={300}>
          <PieChart>
            <Pie
              data={methodData}
              cx="50%"
              cy="50%"
              innerRadius={60}
              outerRadius={100}
              paddingAngle={2}
              dataKey="value"
            >
              {methodData.map((_entry, index) => (
                <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
              ))}
            </Pie>
            <Tooltip contentStyle={{ backgroundColor: '#fff9ec', border: '1px solid #d5c9af', borderRadius: '8px' }} />
          </PieChart>
        </ResponsiveContainer>
        <div className="legend">
          {methodData.map((entry, index) => (
            <div key={entry.name} className="legend-item">
              <span className="legend-color" style={{ backgroundColor: COLORS[index] }}></span>
              {entry.name}: {entry.value}%
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

