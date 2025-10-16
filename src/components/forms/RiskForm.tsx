import { useState } from 'react';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { Select } from '@/components/ui/Select';
import { useEntityStore } from '@/stores/useEntityStore';
import type { Risk } from '@/types';

interface RiskFormProps {
  onSuccess: () => void;
  onCancel: () => void;
  initialData?: Partial<Risk>;
}

export function RiskForm({ onSuccess, onCancel, initialData }: RiskFormProps) {
  const { createRisk, updateRisk } = useEntityStore();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [formData, setFormData] = useState({
    name: initialData?.name || '',
    description: initialData?.description || '',
    notes: initialData?.notes || '',
    risk_type: initialData?.risk_type || 'Technical',
    probability: initialData?.probability || 1,
    severity: initialData?.severity || 1,
  });

  const riskScore = formData.probability * formData.severity;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const riskData = {
        ...formData,
        risk_score: riskScore,
      };

      if (initialData?.metadata?.id) {
        await updateRisk(initialData.metadata.id, riskData);
      } else {
        await createRisk(riskData);
      }

      onSuccess();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          {error}
        </div>
      )}

      <Input
        label="Risk Name"
        required
        value={formData.name}
        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
        placeholder="Enter risk name"
      />

      <Select
        label="Risk Type"
        required
        value={formData.risk_type}
        onChange={(e) => setFormData({ ...formData, risk_type: e.target.value })}
        options={[
          { value: 'Technical', label: 'Technical' },
          { value: 'Safety', label: 'Safety' },
          { value: 'Financial', label: 'Financial' },
          { value: 'Schedule', label: 'Schedule' },
          { value: 'Regulatory', label: 'Regulatory' },
        ]}
      />

      <Textarea
        label="Description"
        required
        value={formData.description}
        onChange={(e) => setFormData({ ...formData, description: e.target.value })}
        placeholder="Describe the risk"
      />

      <div className="grid grid-cols-2 gap-4">
        <div>
          <Select
            label="Probability"
            required
            value={formData.probability.toString()}
            onChange={(e) => setFormData({ ...formData, probability: parseInt(e.target.value) })}
            options={[
              { value: '1', label: '1 - Very Low' },
              { value: '2', label: '2 - Low' },
              { value: '3', label: '3 - Medium' },
              { value: '4', label: '4 - High' },
              { value: '5', label: '5 - Very High' },
            ]}
          />
        </div>

        <div>
          <Select
            label="Severity"
            required
            value={formData.severity.toString()}
            onChange={(e) => setFormData({ ...formData, severity: parseInt(e.target.value) })}
            options={[
              { value: '1', label: '1 - Negligible' },
              { value: '2', label: '2 - Minor' },
              { value: '3', label: '3 - Moderate' },
              { value: '4', label: '4 - Major' },
              { value: '5', label: '5 - Critical' },
            ]}
          />
        </div>
      </div>

      {/* Risk Score Display */}
      <div className="p-4 bg-slate-50 rounded-lg border border-slate-200">
        <div className="text-sm text-slate-600 mb-1">Risk Score</div>
        <div className="flex items-center gap-3">
          <div className={`inline-flex items-center justify-center w-12 h-12 rounded-lg font-bold text-lg ${
            riskScore <= 4 ? 'bg-green-500' :
            riskScore <= 9 ? 'bg-amber-500' :
            riskScore <= 15 ? 'bg-orange-500' :
            'bg-red-500'
          } text-white`}>
            {riskScore}
          </div>
          <div className="text-sm text-slate-600">
            {riskScore <= 4 && 'Low Risk'}
            {riskScore > 4 && riskScore <= 9 && 'Medium Risk'}
            {riskScore > 9 && riskScore <= 15 && 'High Risk'}
            {riskScore > 15 && 'Very High Risk'}
          </div>
        </div>
      </div>

      <Textarea
        label="Notes"
        value={formData.notes}
        onChange={(e) => setFormData({ ...formData, notes: e.target.value })}
        placeholder="Additional notes (optional)"
      />

      <div className="flex justify-end gap-3 pt-4 border-t border-slate-200">
        <button
          type="button"
          onClick={onCancel}
          className="btn btn-secondary"
          disabled={loading}
        >
          Cancel
        </button>
        <button
          type="submit"
          className="btn btn-primary"
          disabled={loading}
        >
          {loading ? 'Saving...' : initialData?.metadata?.id ? 'Update Risk' : 'Create Risk'}
        </button>
      </div>
    </form>
  );
}
