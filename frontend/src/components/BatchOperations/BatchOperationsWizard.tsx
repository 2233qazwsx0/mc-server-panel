import React, { useState } from 'react';

export interface BatchItem {
  id: string;
  name: string;
  type: string;
  selected: boolean;
  data?: Record<string, unknown>;
}

export interface BatchOperation {
  id: string;
  name: string;
  description: string;
  icon?: React.ReactNode;
  action: (items: BatchItem[]) => Promise<void>;
  confirmRequired?: boolean;
  requiresInput?: {
    label: string;
    placeholder: string;
    type: 'text' | 'number' | 'select';
    options?: string[];
  };
}

type WizardStep = 'select' | 'operation' | 'confirm' | 'executing' | 'complete';

interface BatchOperationsWizardProps {
  isOpen: boolean;
  onClose: () => void;
  items: BatchItem[];
  onItemsChange: (items: BatchItem[]) => void;
  operations: BatchOperation[];
  title?: string;
}

export const BatchOperationsWizard: React.FC<BatchOperationsWizardProps> = ({
  isOpen,
  onClose,
  items,
  onItemsChange,
  operations,
  title = 'Batch Operations',
}) => {
  const [step, setStep] = useState<WizardStep>('select');
  const [selectedOperation, setSelectedOperation] = useState<BatchOperation | null>(null);
  const [inputValue, setInputValue] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);

  const selectedItems = items.filter(item => item.selected);
  const allSelected = items.length > 0 && selectedItems.length === items.length;

  const handleSelectAll = () => {
    if (allSelected) {
      onItemsChange(items.map(item => ({ ...item, selected: false })));
    } else {
      onItemsChange(items.map(item => ({ ...item, selected: true })));
    }
  };

  const handleSelectItem = (id: string) => {
    onItemsChange(
      items.map(item =>
        item.id === id ? { ...item, selected: !item.selected } : item
      )
    );
  };

  const handleSelectOperation = (operation: BatchOperation) => {
    setSelectedOperation(operation);
    setInputValue('');
    setError(null);
    setStep(operation.confirmRequired !== false ? 'confirm' : 'confirm');
  };

  const handleExecute = async () => {
    if (!selectedOperation) return;

    if (selectedOperation.requiresInput && !inputValue.trim()) {
      setError('This field is required');
      return;
    }

    setStep('executing');
    setProgress(0);
    setError(null);

    try {
      const itemsToProcess = selectedItems.map(item => ({
        ...item,
        data: selectedOperation.requiresInput
          ? { ...item.data, input: inputValue }
          : item.data,
      }));

      const total = itemsToProcess.length;
      for (let i = 0; i < total; i++) {
        await new Promise(resolve => setTimeout(resolve, 200));
        setProgress(Math.round(((i + 1) / total) * 100));
      }

      await selectedOperation.action(itemsToProcess);
      setStep('complete');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Operation failed');
      setStep('confirm');
    }
  };

  const handleReset = () => {
    setStep('select');
    setSelectedOperation(null);
    setInputValue('');
    setError(null);
    setProgress(0);
  };

  const handleClose = () => {
    handleReset();
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 overflow-y-auto"
      role="dialog"
      aria-modal="true"
      aria-labelledby="batch-wizard-title"
    >
      <div className="fixed inset-0 bg-black/60 backdrop-blur-sm" onClick={handleClose} />

      <div className="relative min-h-full flex items-center justify-center p-4">
        <div className="relative w-full max-w-2xl bg-nether-800 rounded-xl shadow-2xl border border-nether-600 overflow-hidden">
          <div className="px-6 py-4 border-b border-nether-600 flex items-center justify-between">
            <h2 id="batch-wizard-title" className="text-xl font-semibold text-text-primary">
              {title}
            </h2>
            <button
              onClick={handleClose}
              className="p-2 hover:bg-nether-700 rounded-lg transition-colors"
              aria-label="Close"
            >
              <svg className="w-5 h-5 text-text-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div className="p-6">
            {step === 'select' && (
              <div className="space-y-4">
                <div className="flex items-center justify-between mb-4">
                  <span className="text-sm text-text-secondary">
                    {selectedItems.length} of {items.length} selected
                  </span>
                  <button
                    onClick={handleSelectAll}
                    className="text-sm text-mc-green hover:text-mc-green-light transition-colors"
                  >
                    {allSelected ? 'Deselect All' : 'Select All'}
                  </button>
                </div>

                <div className="max-h-64 overflow-y-auto space-y-2">
                  {items.map(item => (
                    <label
                      key={item.id}
                      className="flex items-center gap-3 p-3 bg-nether-700 rounded-lg cursor-pointer hover:bg-nether-600 transition-colors"
                    >
                      <input
                        type="checkbox"
                        checked={item.selected}
                        onChange={() => handleSelectItem(item.id)}
                        className="w-5 h-5 rounded border-nether-600 bg-nether-800 text-mc-green focus:ring-mc-green focus:ring-offset-0"
                      />
                      <div className="flex-1">
                        <div className="font-medium text-text-primary">{item.name}</div>
                        <div className="text-sm text-text-secondary">{item.type}</div>
                      </div>
                    </label>
                  ))}
                </div>

                {selectedItems.length > 0 && (
                  <div className="pt-4 border-t border-nether-600">
                    <p className="text-sm text-text-secondary mb-3">Choose an operation:</p>
                    <div className="grid gap-2">
                      {operations.map(operation => (
                        <button
                          key={operation.id}
                          onClick={() => handleSelectOperation(operation)}
                          className="flex items-center gap-3 p-4 bg-nether-700 hover:bg-nether-600 rounded-lg transition-colors text-left"
                        >
                          <div className="p-2 bg-mc-green/20 rounded-lg">
                            {operation.icon || (
                              <svg className="w-5 h-5 text-mc-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                              </svg>
                            )}
                          </div>
                          <div className="flex-1">
                            <div className="font-medium text-text-primary">{operation.name}</div>
                            <div className="text-sm text-text-secondary">{operation.description}</div>
                          </div>
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {step === 'confirm' && selectedOperation && (
              <div className="space-y-4">
                <div className="p-4 bg-mc-green/10 border border-mc-green/30 rounded-lg">
                  <div className="flex items-center gap-3 mb-2">
                    <div className="p-2 bg-mc-green/20 rounded-lg">
                      {selectedOperation.icon || (
                        <svg className="w-5 h-5 text-mc-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                        </svg>
                      )}
                    </div>
                    <div>
                      <h3 className="font-semibold text-text-primary">{selectedOperation.name}</h3>
                      <p className="text-sm text-text-secondary">{selectedOperation.description}</p>
                    </div>
                  </div>
                </div>

                <div className="p-3 bg-nether-700 rounded-lg">
                  <p className="text-sm text-text-secondary mb-2">Affected items:</p>
                  <ul className="space-y-1">
                    {selectedItems.map(item => (
                      <li key={item.id} className="text-sm text-text-primary flex items-center gap-2">
                        <svg className="w-4 h-4 text-mc-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                        {item.name}
                      </li>
                    ))}
                  </ul>
                </div>

                {selectedOperation.requiresInput && (
                  <div className="space-y-2">
                    <label className="block text-sm font-medium text-text-primary">
                      {selectedOperation.requiresInput.label}
                    </label>
                    {selectedOperation.requiresInput.type === 'select' ? (
                      <select
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        className="w-full px-4 py-2 bg-nether-700 border border-nether-600 rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-mc-green"
                      >
                        <option value="">{selectedOperation.requiresInput.placeholder}</option>
                        {selectedOperation.requiresInput.options?.map(option => (
                          <option key={option} value={option}>{option}</option>
                        ))}
                      </select>
                    ) : (
                      <input
                        type={selectedOperation.requiresInput.type}
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        placeholder={selectedOperation.requiresInput.placeholder}
                        className="w-full px-4 py-2 bg-nether-700 border border-nether-600 rounded-lg text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-mc-green"
                      />
                    )}
                  </div>
                )}

                {error && (
                  <div className="p-3 bg-status-error/10 border border-status-error/30 rounded-lg text-status-error text-sm">
                    {error}
                  </div>
                )}

                <div className="flex gap-3 pt-4">
                  <button
                    onClick={() => setStep('select')}
                    className="flex-1 px-4 py-2 bg-nether-700 hover:bg-nether-600 text-text-primary rounded-lg transition-colors"
                  >
                    Back
                  </button>
                  <button
                    onClick={handleExecute}
                    className="flex-1 px-4 py-2 bg-mc-green hover:bg-mc-green-light text-white rounded-lg transition-colors"
                  >
                    Confirm
                  </button>
                </div>
              </div>
            )}

            {step === 'executing' && (
              <div className="py-8 text-center">
                <div className="inline-flex items-center justify-center w-16 h-16 mb-4">
                  <svg className="animate-spin w-16 h-16 text-mc-green" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold text-text-primary mb-2">Processing...</h3>
                <div className="w-full bg-nether-700 rounded-full h-2 mb-2">
                  <div
                    className="bg-mc-green h-2 rounded-full transition-all duration-300"
                    style={{ width: `${progress}%` }}
                  />
                </div>
                <p className="text-sm text-text-secondary">{progress}% complete</p>
              </div>
            )}

            {step === 'complete' && (
              <div className="py-8 text-center">
                <div className="inline-flex items-center justify-center w-16 h-16 mb-4 bg-mc-green/20 rounded-full">
                  <svg className="w-8 h-8 text-mc-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold text-text-primary mb-2">Operation Complete!</h3>
                <p className="text-sm text-text-secondary mb-6">
                  Successfully processed {selectedItems.length} items
                </p>
                <button
                  onClick={handleReset}
                  className="px-6 py-2 bg-mc-green hover:bg-mc-green-light text-white rounded-lg transition-colors"
                >
                  Done
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
