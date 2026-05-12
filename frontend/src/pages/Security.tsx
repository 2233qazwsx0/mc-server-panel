import { useState, useEffect } from 'react';
import { SECURITY_TABS } from '../types/security';
import { IpFilterPanel } from '../components/security/IpFilterPanel';
import { DdosProtectionPanel } from '../components/security/DdosProtectionPanel';
import { BruteForcePanel } from '../components/security/BruteForcePanel';
import { SslCertPanel } from '../components/security/SslCertPanel';
import { TotpPanel } from '../components/security/TotpPanel';
import { AuditLogPanel } from '../components/security/AuditLogPanel';
import { SessionPanel } from '../components/security/SessionPanel';
import { ApiKeyPanel } from '../components/security/ApiKeyPanel';
import { EncryptionPanel } from '../components/security/EncryptionPanel';
import { SecurityScanPanel } from '../components/security/SecurityScanPanel';

const API_BASE = 'http://localhost:8080';

export function Security() {
  const [activeTab, setActiveTab] = useState('ip-filter');

  const renderPanel = () => {
    switch (activeTab) {
      case 'ip-filter':
        return <IpFilterPanel apiBase={API_BASE} />;
      case 'ddos':
        return <DdosProtectionPanel apiBase={API_BASE} />;
      case 'bruteforce':
        return <BruteForcePanel apiBase={API_BASE} />;
      case 'ssl':
        return <SslCertPanel apiBase={API_BASE} />;
      case '2fa':
        return <TotpPanel apiBase={API_BASE} />;
      case 'audit':
        return <AuditLogPanel apiBase={API_BASE} />;
      case 'sessions':
        return <SessionPanel apiBase={API_BASE} />;
      case 'apikeys':
        return <ApiKeyPanel apiBase={API_BASE} />;
      case 'encryption':
        return <EncryptionPanel apiBase={API_BASE} />;
      case 'baseline':
        return <SecurityScanPanel apiBase={API_BASE} />;
      default:
        return null;
    }
  };

  return (
    <div className="h-full flex flex-col">
      <div className="bg-gray-800 border-b border-gray-700">
        <div className="flex space-x-1 px-4 overflow-x-auto">
          {SECURITY_TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium whitespace-nowrap transition-colors ${
                activeTab === tab.id
                  ? 'text-blue-400 border-b-2 border-blue-400'
                  : 'text-gray-400 hover:text-gray-200'
              }`}
            >
              <span className="mr-2">{tab.icon}</span>
              {tab.label}
            </button>
          ))}
        </div>
      </div>
      <div className="flex-1 overflow-auto p-6 bg-gray-900">
        {renderPanel()}
      </div>
    </div>
  );
}
