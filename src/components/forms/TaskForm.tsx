import { useState } from 'react';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { Select } from '@/components/ui/Select';
import { useEntityStore } from '@/stores/useEntityStore';
import type { Task } from '@/types';

interface TaskFormProps {
  onSuccess: () => void;
  onCancel: () => void;
  initialData?: Partial<Task>;
}

export function TaskForm({ onSuccess, onCancel, initialData }: TaskFormProps) {
  const { createTask, updateTask } = useEntityStore();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [formData, setFormData] = useState({
    name: initialData?.name || '',
    description: initialData?.description || '',
    notes: initialData?.notes || '',
    scheduled_start: initialData?.scheduled_start || new Date().toISOString().split('T')[0],
    deadline: initialData?.deadline || '',
    task_type: initialData?.task_type || 'EffortDriven',
    scheduling_mode: initialData?.scheduling_mode || 'Automatic',
    percent_complete: initialData?.percent_complete || 0,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const taskData = {
        ...formData,
        assigned_resources: [],
        dependencies: [],
        is_critical_path: false,
      };

      if (initialData?.metadata?.id) {
        await updateTask(initialData.metadata.id, taskData);
      } else {
        await createTask(taskData);
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
        label="Task Name"
        required
        value={formData.name}
        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
        placeholder="Enter task name"
      />

      <Textarea
        label="Description"
        required
        value={formData.description}
        onChange={(e) => setFormData({ ...formData, description: e.target.value })}
        placeholder="Describe the task"
      />

      <Textarea
        label="Notes"
        value={formData.notes}
        onChange={(e) => setFormData({ ...formData, notes: e.target.value })}
        placeholder="Additional notes (optional)"
      />

      <div className="grid grid-cols-2 gap-4">
        <Input
          label="Scheduled Start"
          type="date"
          required
          value={formData.scheduled_start}
          onChange={(e) => setFormData({ ...formData, scheduled_start: e.target.value })}
        />

        <Input
          label="Deadline"
          type="date"
          required
          value={formData.deadline}
          onChange={(e) => setFormData({ ...formData, deadline: e.target.value })}
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <Select
          label="Task Type"
          required
          value={formData.task_type}
          onChange={(e) => setFormData({ ...formData, task_type: e.target.value as any })}
          options={[
            { value: 'EffortDriven', label: 'Effort Driven' },
            { value: 'DurationDriven', label: 'Duration Driven' },
            { value: 'WorkDriven', label: 'Work Driven' },
          ]}
        />

        <Select
          label="Scheduling Mode"
          required
          value={formData.scheduling_mode}
          onChange={(e) => setFormData({ ...formData, scheduling_mode: e.target.value as any })}
          options={[
            { value: 'Automatic', label: 'Automatic' },
            { value: 'Manual', label: 'Manual' },
          ]}
        />
      </div>

      <Input
        label="Percent Complete"
        type="number"
        min="0"
        max="100"
        value={formData.percent_complete * 100}
        onChange={(e) => setFormData({ ...formData, percent_complete: parseFloat(e.target.value) / 100 })}
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
          {loading ? 'Saving...' : initialData?.metadata?.id ? 'Update Task' : 'Create Task'}
        </button>
      </div>
    </form>
  );
}
