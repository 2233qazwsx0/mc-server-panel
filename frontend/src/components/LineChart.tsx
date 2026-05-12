import { LineChart as RechartsLineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, AreaChart, Area } from 'recharts';
import { MetricPoint } from '@/types';

interface LineChartProps {
  data: MetricPoint[];
  color: string;
  title?: string;
  unit?: string;
}

export function LineChart({ data, color, title, unit }: LineChartProps) {
  return (
    <div className="chart-container">
      {title && (
        <h3 className="chart-title">{title}</h3>
      )}
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data}>
            <defs>
              <linearGradient id={`color-${color.replace('#', '')}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={color} stopOpacity={0.3} />
                <stop offset="95%" stopColor={color} stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="#333333" />
            <XAxis
              dataKey="time"
              stroke="#666666"
              tick={{ fontSize: 10, fontFamily: 'Fira Code' }}
              tickLine={false}
              axisLine={false}
            />
            <YAxis
              stroke="#666666"
              tick={{ fontSize: 10, fontFamily: 'Fira Code' }}
              tickLine={false}
              axisLine={false}
              unit={unit}
              domain={[0, 100]}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: '#1A1A1A',
                border: '1px solid #333333',
                borderRadius: '8px',
                fontFamily: 'Fira Code',
              }}
              labelStyle={{ color: '#A0A0A0', fontFamily: 'Fira Code' }}
              itemStyle={{ color, fontFamily: 'Fira Code' }}
              cursor={{ stroke: '#5D7C15', strokeWidth: 1 }}
            />
            <Area
              type="monotone"
              dataKey="value"
              stroke={color}
              strokeWidth={2}
              fillOpacity={1}
              fill={`url(#color-${color.replace('#', '')})`}
              animationDuration={500}
              animationEasing="ease-out"
              dot={false}
              activeDot={{
                r: 4,
                fill: color,
                stroke: '#1A1A1A',
                strokeWidth: 2,
              }}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
