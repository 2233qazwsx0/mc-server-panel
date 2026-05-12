import React, { useState, useEffect } from 'react';
import { Shield, User, Users, Trash2, Edit, Plus } from 'lucide-react';
import { AclRule, AclPrincipal, ApiResponse } from './types';

const AclManager: React.FC = () => {
  const [rules, setRules] = useState<AclRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newRule, setNewRule] = useState({
    path: '',
    principal_kind: 'user',
    principal_id: '',
    principal_name: '',
    permissions: [] as string[],
    recursive: false
  });

  useEffect(() => {
    loadRules();
  }, []);

  const loadRules = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/files/acl/rules');
      const data: ApiResponse<AclRule[]> = await response.json();
      if (data.success && data.data) {
        setRules(data.data);
      }
    } catch (err) {
      console.error('Failed to load ACL rules:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAddRule = async () => {
    try {
      const response = await fetch('/api/files/acl/rules', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path: newRule.path,
          principal: {
            kind: newRule.principal_kind,
            id: newRule.principal_id,
            name: newRule.principal_name
          },
          permissions: newRule.permissions,
          recursive: newRule.recursive
        })
      });
      
      const data: ApiResponse<AclRule> = await response.json();
      if (data.success) {
        setShowAddForm(false);
        loadRules();
        setNewRule({
          path: '',
          principal_kind: 'user',
          principal_id: '',
          principal_name: '',
          permissions: [],
          recursive: false
        });
      }
    } catch (err) {
      console.error('Failed to add ACL rule:', err);
    }
  };

  const handleDeleteRule = async (id: string) => {
    try {
      const response = await fetch(`/api/files/acl/rules/${id}`, {
        method: 'DELETE'
      });
      const data: ApiResponse<boolean> = await response.json();
      if (data.success) {
        loadRules();
      }
    } catch (err) {
      console.error('Failed to delete ACL rule:', err);
    }
  };

  const togglePermission = (perm: string) => {
    setNewRule(prev => ({
      ...prev,
      permissions: prev.permissions.includes(perm)
        ? prev.permissions.filter(p => p !== perm)
        : [...prev.permissions, perm]
    }));
  };

  if (loading) {
    return <div className="flex items-center justify-center h-full text-gray-400">Loading...</div>;
  }

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Shield className="w-5 h-5 text-green-400" />
            <h2 className="text-lg font-semibold text-white">Access Control List</h2>
          </div>
          <button
            onClick={() => setShowAddForm(true)}
            className="px-4 py-2 bg-green-600 text-white rounded text-sm hover:bg-green-700"
          >
            <Plus className="w-4 h-4 inline mr-1" />
            Add Rule
          </button>
        </div>
      </div>

      {showAddForm && (
        <div className="p-4 bg-gray-800 border-b border-gray-700">
          <h3 className="text-white font-medium mb-4">New ACL Rule</h3>
          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Path</label>
              <input
                type="text"
                value={newRule.path}
                onChange={(e) => setNewRule({...newRule, path: e.target.value})}
                placeholder="/plugins/MyPlugin/config.yml"
                className="w-full px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Principal Type</label>
              <select
                value={newRule.principal_kind}
                onChange={(e) => setNewRule({...newRule, principal_kind: e.target.value})}
                className="w-full px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
              >
                <option value="user">User</option>
                <option value="group">Group</option>
                <option value="role">Role</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Principal ID</label>
              <input
                type="text"
                value={newRule.principal_id}
                onChange={(e) => setNewRule({...newRule, principal_id: e.target.value})}
                placeholder="user-123"
                className="w-full px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Principal Name</label>
              <input
                type="text"
                value={newRule.principal_name}
                onChange={(e) => setNewRule({...newRule, principal_name: e.target.value})}
                placeholder="John Doe"
                className="w-full px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
              />
            </div>
          </div>

          <div className="mb-4">
            <label className="block text-sm text-gray-400 mb-2">Permissions</label>
            <div className="flex space-x-2">
              {['read', 'write', 'delete', 'execute', 'admin'].map(perm => (
                <button
                  key={perm}
                  onClick={() => togglePermission(perm)}
                  className={`px-3 py-1 rounded text-sm ${
                    newRule.permissions.includes(perm)
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 text-gray-300'
                  }`}
                >
                  {perm}
                </button>
              ))}
            </div>
          </div>

          <label className="flex items-center space-x-2 mb-4">
            <input
              type="checkbox"
              checked={newRule.recursive}
              onChange={(e) => setNewRule({...newRule, recursive: e.target.checked})}
              className="rounded"
            />
            <span className="text-sm text-gray-300">Apply recursively to subdirectories</span>
          </label>

          <div className="flex space-x-2">
            <button
              onClick={handleAddRule}
              className="px-4 py-2 bg-green-600 text-white rounded text-sm"
            >
              Add Rule
            </button>
            <button
              onClick={() => setShowAddForm(false)}
              className="px-4 py-2 bg-gray-700 text-white rounded text-sm"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      <div className="flex-1 overflow-auto p-4">
        {rules.length === 0 ? (
          <div className="text-center text-gray-400 py-8">No ACL rules defined</div>
        ) : (
          <div className="space-y-2">
            {rules.map(rule => (
              <div key={rule.id} className="bg-gray-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center space-x-2">
                    {rule.principal.kind === 'user' ? (
                      <User className="w-4 h-4 text-blue-400" />
                    ) : (
                      <Users className="w-4 h-4 text-purple-400" />
                    )}
                    <span className="text-white font-medium">{rule.principal.name}</span>
                    <span className="text-gray-500 text-sm">({rule.principal.id})</span>
                  </div>
                  <button
                    onClick={() => handleDeleteRule(rule.id)}
                    className="text-red-400 hover:text-red-300"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
                <div className="text-sm text-gray-400 mb-2">Path: {rule.path}</div>
                <div className="flex items-center space-x-2">
                  {rule.permissions.map(perm => (
                    <span key={perm} className="px-2 py-1 bg-gray-700 rounded text-xs text-gray-300">
                      {perm}
                    </span>
                  ))}
                  {rule.recursive && (
                    <span className="px-2 py-1 bg-gray-600 rounded text-xs text-gray-400">
                      recursive
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default AclManager;
