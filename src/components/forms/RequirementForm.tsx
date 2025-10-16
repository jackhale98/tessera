import { useState } from 'react';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { Select } from '@/components/ui/Select';
import { useEntityStore } from '@/stores/useEntityStore';
import type { Requirement } from '@/types';

interface RequirementFormProps {
  onSuccess: () => void;
  onCancel: () => void;
  initialData?: Partial<Requirement>;
}

export function RequirementForm({ onSuccess, onCancel, initialData }: RequirementFormProps) {
  const { createRequirement, updateRequirement } = useEntityStore();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [formData, setFormData] = useState({
    name: initialData?.name || '',
    description: initialData?.description || '',
    notes: initialData?.notes || '',
    requirement_type: initialData?.requirement_type || 'User Requirement',
    rationale: initialData?.rationale || '',
    source: initialData?.source || '',
    verification_method: initialData?.verification_method || '',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      if (initialData?.metadata?.id) {
        await updateRequirement(initialData.metadata.id, formData);
      } else {
        await createRequirement(formData);
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
        label="Requirement Name"
        required
        value={formData.name}
        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
        placeholder="Enter requirement name"
      />

      <Select
        label="Requirement Type"
        required
        value={formData.requirement_type}
        onChange={(e) => setFormData({ ...formData, requirement_type: e.target.value })}
        options={[
          { value: 'User Requirement', label: 'User Requirement' },
          { value: 'System Requirement', label: 'System Requirement' },
          { value: 'Design Requirement', label: 'Design Requirement' },
          { value: 'Software Requirement', label: 'Software Requirement' },
          { value: 'Safety Requirement', label: 'Safety Requirement' },
        ]}
      />

      <Textarea
        label="Description"
        required
        value={formData.description}
        onChange={(e) => setFormData({ ...formData, description: e.target.value })}
        placeholder="Describe the requirement"
      />

      <Textarea
        label="Rationale"
        value={formData.rationale}
        onChange={(e) => setFormData({ ...formData, rationale: e.target.value })}
        placeholder="Why is this requirement needed?"
      />

      <Input
        label="Source"
        value={formData.source}
        onChange={(e) => setFormData({ ...formData, source: e.target.value })}
        placeholder="Source of the requirement (e.g., Customer, Regulation)"
      />

      <Input
        label="Verification Method"
        value={formData.verification_method}
        onChange={(e) => setFormData({ ...formData, verification_method: e.target.value })}
        placeholder="How will this be verified?"
      />

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
          {loading ? 'Saving...' : initialData?.metadata?.id ? 'Update Requirement' : 'Create Requirement'}
        </button>
      </div>
    </form>
  );
}
