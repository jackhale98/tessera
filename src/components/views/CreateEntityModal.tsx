import { Modal } from '@/components/ui/Modal';
import { TaskForm } from '@/components/forms/TaskForm';
import { RequirementForm } from '@/components/forms/RequirementForm';
import { RiskForm } from '@/components/forms/RiskForm';

interface CreateEntityModalProps {
  isOpen: boolean;
  onClose: () => void;
  entityType: 'task' | 'requirement' | 'risk' | null;
}

export function CreateEntityModal({ isOpen, onClose, entityType }: CreateEntityModalProps) {
  const handleSuccess = () => {
    onClose();
    // Could add a toast notification here
  };

  const getTitle = () => {
    switch (entityType) {
      case 'task':
        return 'Create New Task';
      case 'requirement':
        return 'Create New Requirement';
      case 'risk':
        return 'Create New Risk';
      default:
        return 'Create New Entity';
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={getTitle()} size="lg">
      {entityType === 'task' && <TaskForm onSuccess={handleSuccess} onCancel={onClose} />}
      {entityType === 'requirement' && <RequirementForm onSuccess={handleSuccess} onCancel={onClose} />}
      {entityType === 'risk' && <RiskForm onSuccess={handleSuccess} onCancel={onClose} />}
    </Modal>
  );
}
