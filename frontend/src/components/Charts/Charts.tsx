import React from 'react';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';

const CHART_COLORS = ['#5D7C15', '#DEA584', '#4FC3F7', '#E53935', '#9C6ADE', '#FFD93D'];

interface ChartTooltipProps {
  active?: boolean;
  payload?: Array<{
    name: string;
    value: number;
    color: string;
  }>;
  label?: string;
}

const CustomTooltip: React.FC<ChartTooltipProps> = ({ active, payload, label }) => {
  if (active && payload && payload.length) {
    return (
      <div className="bg-nether-800 border border-nether-600 rounded-lg p-3 shadow-xl">
        <p className="text-text-primary font-medium mb-2">{label}</p>
        {payload.map((entry, index) => (
          <div key={index} className="flex items-center gap-2 text-sm">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-text-secondary">{entry.name}:</span>
            <span className="text-text-primary font-medium">{entry.value}</span>
          </div>
        ))}
      </div>
    );
  }
  return null;
};

interface PerformanceChartProps {
  data: Array<{
    time: string;
    cpu: number;
    memory: number;
    tps: number;
  }>;
  height?: number;
}

export const PerformanceChart: React.FC<PerformanceChartProps> = ({
  data,
  height = 300,
}) => {
  return (
    <div className="chart-container">
      <h3 className="chart-title font-semibold">Performance Metrics</h3>
      <ResponsiveContainer width="100%" height={height}>
        <AreaChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <defs>
            <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#5D7C15" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#5D7C15" stopOpacity={0} />
            </linearGradient>
            <linearGradient id="memoryGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#DEA584" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#DEA584" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="#333" />
          <XAxis dataKey="time" stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <YAxis stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <Tooltip content={<CustomTooltip />} />
          <Legend />
          <Area
            type="monotone"
            dataKey="cpu"
            stroke="#5D7C15"
            fill="url(#cpuGradient)"
            strokeWidth={2}
            name="CPU %"
          />
          <Area
            type="monotone"
            dataKey="memory"
            stroke="#DEA584"
            fill="url(#memoryGradient)"
            strokeWidth={2}
            name="Memory %"
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};

interface TpsChartProps {
  data: Array<{
    time: string;
    tps: number;
  }>;
  height?: number;
}

export const TpsChart: React.FC<TpsChartProps> = ({ data, height = 200 }) => {
  return (
    <div className="chart-container">
      <h3 className="chart-title font-semibold">Server TPS</h3>
      <ResponsiveContainer width="100%" height={height}>
        <LineChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#333" />
          <XAxis dataKey="time" stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <YAxis domain={[0, 20]} stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <Tooltip content={<CustomTooltip />} />
          <Line
            type="monotone"
            dataKey="tps"
            stroke="#5D7C15"
            strokeWidth={2}
            dot={{ fill: '#5D7C15', r: 3 }}
            activeDot={{ r: 5 }}
            name="TPS"
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};

interface PlayersOnlineChartProps {
  data: Array<{
    time: string;
    players: number;
    maxPlayers: number;
  }>;
  height?: number;
}

export const PlayersOnlineChart: React.FC<PlayersOnlineChartProps> = ({
  data,
  height = 200,
}) => {
  return (
    <div className="chart-container">
      <h3 className="chart-title font-semibold">Players Online</h3>
      <ResponsiveContainer width="100%" height={height}>
        <BarChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#333" />
          <XAxis dataKey="time" stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <YAxis stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <Tooltip content={<CustomTooltip />} />
          <Bar dataKey="players" fill="#5D7C15" radius={[4, 4, 0, 0]} name="Players" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
};

interface ResourceUsageChartProps {
  data: Array<{
    name: string;
    value: number;
  }>[];
  labels: string[];
  height?: number;
}

export const ResourceUsageChart: React.FC<ResourceUsageChartProps> = ({
  data,
  labels,
  height = 200,
}) => {
  return (
    <div className="chart-container">
      <h3 className="chart-title font-semibold">Resource Distribution</h3>
      <ResponsiveContainer width="100%" height={height}>
        <PieChart>
          <Pie
            data={data[0]}
            cx="50%"
            cy="50%"
            innerRadius={40}
            outerRadius={80}
            paddingAngle={2}
            dataKey="value"
          >
            {data[0].map((entry, index) => (
              <Cell key={`cell-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
            ))}
          </Pie>
          <Tooltip content={<CustomTooltip />} />
          <Legend
            formatter={(value) => <span className="text-text-secondary text-sm">{value}</span>}
          />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
};

interface WorldAnalysisChartProps {
  data: Array<{
    chunk: string;
    entities: number;
    tiles: number;
  }>;
  height?: number;
}

export const WorldAnalysisChart: React.FC<WorldAnalysisChartProps> = ({
  data,
  height = 250,
}) => {
  return (
    <div className="chart-container">
      <h3 className="chart-title font-semibold">World Analysis</h3>
      <ResponsiveContainer width="100%" height={height}>
        <BarChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#333" />
          <XAxis dataKey="chunk" stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <YAxis stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <Tooltip content={<CustomTooltip />} />
          <Legend />
          <Bar dataKey="entities" fill="#5D7C15" radius={[4, 4, 0, 0]} name="Entities" />
          <Bar dataKey="tiles" fill="#DEA584" radius={[4, 4, 0, 0]} name="Tile Entities" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
};

interface NetworkChartProps {
  data: Array<{
    time: string;
    bytesIn: number;
    bytesOut: number;
  }>;
  height?: number;
}

export const NetworkChart: React.FC<NetworkChartProps> = ({ data, height = 200 }) => {
  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="chart-container">
      <h3 className="chart-title font-semibold">Network Traffic</h3>
      <ResponsiveContainer width="100%" height={height}>
        <AreaChart data={data} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <defs>
            <linearGradient id="bytesInGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#4FC3F7" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#4FC3F7" stopOpacity={0} />
            </linearGradient>
            <linearGradient id="bytesOutGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#9C6ADE" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#9C6ADE" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="#333" />
          <XAxis dataKey="time" stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} />
          <YAxis stroke="#666" tick={{ fill: '#A0A0A0', fontSize: 12 }} tickFormatter={formatBytes} />
          <Tooltip content={<CustomTooltip />} formatter={(value: number) => formatBytes(value)} />
          <Legend />
          <Area
            type="monotone"
            dataKey="bytesIn"
            stroke="#4FC3F7"
            fill="url(#bytesInGradient)"
            strokeWidth={2}
            name="Inbound"
          />
          <Area
            type="monotone"
            dataKey="bytesOut"
            stroke="#9C6ADE"
            fill="url(#bytesOutGradient)"
            strokeWidth={2}
            name="Outbound"
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};

export { CHART_COLORS };
